// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
#[derive(Clone, Copy, Debug)]
pub struct TokioRuntimeMonitorMetricNames {
    pub tokio_runtime_injection_queue_depth: &'static str,
    pub tokio_runtime_max_busy_duration: &'static str,
    pub tokio_runtime_max_local_queue_depth: &'static str,
    pub tokio_runtime_max_local_schedule_count: &'static str,
    pub tokio_runtime_max_noop_count: &'static str,
    pub tokio_runtime_max_overflow_count: &'static str,
    pub tokio_runtime_max_park_count: &'static str,
    pub tokio_runtime_max_polls_count: &'static str,
    pub tokio_runtime_max_steal_count: &'static str,
    pub tokio_runtime_min_busy_duration: &'static str,
    pub tokio_runtime_min_local_queue_depth: &'static str,
    pub tokio_runtime_min_local_schedule_count: &'static str,
    pub tokio_runtime_min_noop_count: &'static str,
    pub tokio_runtime_min_overflow_count: &'static str,
    pub tokio_runtime_min_park_count: &'static str,
    pub tokio_runtime_min_polls_count: &'static str,
    pub tokio_runtime_min_steal_count: &'static str,
    pub tokio_runtime_num_remote_schedules: &'static str,
    pub tokio_runtime_total_busy_duration: &'static str,
    pub tokio_runtime_total_local_queue_depth: &'static str,
    pub tokio_runtime_total_local_schedule_count: &'static str,
    pub tokio_runtime_total_noop_count: &'static str,
    pub tokio_runtime_total_overflow_count: &'static str,
    pub tokio_runtime_total_park_count: &'static str,
    pub tokio_runtime_total_polls_count: &'static str,
    pub tokio_runtime_total_steal_count: &'static str,
    pub tokio_runtime_workers_count: &'static str,
    pub tokio_runtime_elapsed_duration: &'static str,
    pub tokio_runtime_worker_busy_duration_micros: &'static str,
}

impl Default for TokioRuntimeMonitorMetricNames {
    fn default() -> Self {
        Self {
            tokio_runtime_injection_queue_depth: "tokio_runtime_injection_queue_depth",
            tokio_runtime_max_busy_duration: "tokio_runtime_max_busy_duration",
            tokio_runtime_max_local_queue_depth: "tokio_runtime_max_local_queue_depth",
            tokio_runtime_max_local_schedule_count: "tokio_runtime_max_local_schedule_count",
            tokio_runtime_max_noop_count: "tokio_runtime_max_noop_count",
            tokio_runtime_max_overflow_count: "tokio_runtime_max_overflow_count",
            tokio_runtime_max_park_count: "tokio_runtime_max_park_count",
            tokio_runtime_max_polls_count: "tokio_runtime_max_polls_count",
            tokio_runtime_max_steal_count: "tokio_runtime_max_steal_count",
            tokio_runtime_min_busy_duration: "tokio_runtime_min_busy_duration",
            tokio_runtime_min_local_queue_depth: "tokio_runtime_min_local_queue_depth",
            tokio_runtime_min_local_schedule_count: "tokio_runtime_min_local_schedule_count",
            tokio_runtime_min_noop_count: "tokio_runtime_min_noop_count",
            tokio_runtime_min_overflow_count: "tokio_runtime_min_overflow_count",
            tokio_runtime_min_park_count: "tokio_runtime_min_park_count",
            tokio_runtime_min_polls_count: "tokio_runtime_min_polls_count",
            tokio_runtime_min_steal_count: "tokio_runtime_min_steal_count",
            tokio_runtime_num_remote_schedules: "tokio_runtime_num_remote_schedules",
            tokio_runtime_total_busy_duration: "tokio_runtime_total_busy_duration",
            tokio_runtime_total_local_queue_depth: "tokio_runtime_total_local_queue_depth",
            tokio_runtime_total_local_schedule_count: "tokio_runtime_total_local_schedule_count",
            tokio_runtime_total_noop_count: "tokio_runtime_total_noop_count",
            tokio_runtime_total_overflow_count: "tokio_runtime_total_overflow_count",
            tokio_runtime_total_park_count: "tokio_runtime_total_park_count",
            tokio_runtime_total_polls_count: "tokio_runtime_total_polls_count",
            tokio_runtime_total_steal_count: "tokio_runtime_total_steal_count",
            tokio_runtime_workers_count: "tokio_runtime_workers_count",
            tokio_runtime_elapsed_duration: "tokio_runtime_elapsed_duration",
            tokio_runtime_worker_busy_duration_micros: "tokio_runtime_worker_busy_duration",
        }
    }
}

#[cfg(all(tokio_unstable))]
pub fn start_runtime_monitoring(
    runtime_handle: tokio::runtime::Handle,
    tokio_runtime_metric_names: TokioRuntimeMonitorMetricNames,
    runtime_name: &'static str,
) {
    use std::time::Duration;

    use metrics::{describe_gauge, gauge, Unit};

    struct PerWorkerMetrics {
        labels: Vec<[(String, String); 2]>,
    }

    impl PerWorkerMetrics {
        fn new(
            tokio_runtime_metrics: &tokio::runtime::RuntimeMetrics,
            threadpool_name: &'static str,
        ) -> Self {
            let labels = (0..tokio_runtime_metrics.num_workers())
                .map(|worker| {
                    [
                        ("threadpool".to_string(), threadpool_name.to_string()),
                        ("worker".to_string(), worker.to_string()),
                    ]
                })
                .collect();
            Self { labels }
        }

        fn record_metrics(
            &mut self,
            tokio_runtime_metrics: tokio::runtime::RuntimeMetrics,
            metric_names: &TokioRuntimeMonitorMetricNames,
        ) {
            let num_workers = self.labels.len();
            for worker in 0..num_workers {
                let current_duration = tokio_runtime_metrics.worker_total_busy_duration(worker);
                gauge!(
                    metric_names.tokio_runtime_worker_busy_duration_micros,
                    current_duration.as_micros() as f64,
                    &self.labels[worker]
                );
            }
        }
    }

    let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&runtime_handle);
    let mut per_worker_metrics = PerWorkerMetrics::new(&runtime_handle.metrics(), runtime_name);
    let inner_handle = runtime_handle.clone();

    describe_gauges(&tokio_runtime_metric_names);

    runtime_handle.spawn(async move {
        let labels = [("threadpool", runtime_name)];
        for interval in runtime_monitor.intervals() {
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_injection_queue_depth,
                interval.injection_queue_depth as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_busy_duration,
                interval.max_busy_duration.as_micros() as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_local_queue_depth,
                interval.max_local_queue_depth as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_local_schedule_count,
                interval.max_local_schedule_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_noop_count,
                interval.max_noop_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_overflow_count,
                interval.max_overflow_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_park_count,
                interval.max_park_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_polls_count,
                interval.max_polls_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_max_steal_count,
                interval.max_steal_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_busy_duration,
                interval.min_busy_duration.as_micros() as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_local_queue_depth,
                interval.min_local_queue_depth as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_local_schedule_count,
                interval.min_local_schedule_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_noop_count,
                interval.min_noop_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_overflow_count,
                interval.min_overflow_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_park_count,
                interval.min_park_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_polls_count,
                interval.min_polls_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_min_steal_count,
                interval.min_steal_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_num_remote_schedules,
                interval.num_remote_schedules as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_busy_duration,
                interval.total_busy_duration.as_micros() as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_local_queue_depth,
                interval.total_local_queue_depth as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_local_schedule_count,
                interval.total_local_schedule_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_noop_count,
                interval.total_noop_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_overflow_count,
                interval.total_overflow_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_park_count,
                interval.total_park_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_polls_count,
                interval.total_polls_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_total_steal_count,
                interval.total_steal_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_workers_count,
                interval.workers_count as f64,
                &labels
            );
            gauge!(
                tokio_runtime_metric_names.tokio_runtime_elapsed_duration,
                interval.elapsed.as_micros() as f64,
                &labels
            );
            per_worker_metrics.record_metrics(inner_handle.metrics(), &tokio_runtime_metric_names);

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    fn describe_gauges(tokio_runtime_metric_names: &TokioRuntimeMonitorMetricNames) {
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_injection_queue_depth,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.injection_queue_depth"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_busy_duration,
        Unit::Microseconds,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_busy_duration"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_local_queue_depth,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_local_queue_depth"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_local_schedule_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_local_schedule_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_noop_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_noop_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_overflow_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_overflow_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_park_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_park_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_polls_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_polls_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_max_steal_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_steal_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_busy_duration,
        Unit::Microseconds,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_busy_duration"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_local_queue_depth,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_local_queue_depth"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_local_schedule_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_local_schedule_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_noop_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_noop_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_overflow_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_overflow_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_park_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_park_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_polls_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_polls_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_min_steal_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.min_steal_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_num_remote_schedules,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.num_remote_schedules"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_busy_duration,
        Unit::Microseconds,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_busy_duration"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_local_queue_depth,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_local_queue_depth"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_local_schedule_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_local_schedule_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_noop_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_noop_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_overflow_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_overflow_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_park_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_park_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_polls_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_polls_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_total_steal_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_steal_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_workers_count,
        Unit::Count,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.workers_count"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_elapsed_duration,
        Unit::Microseconds,
        "See https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.elapsed"
    );
        describe_gauge!(
        tokio_runtime_metric_names.tokio_runtime_worker_busy_duration_micros,
        Unit::Microseconds,
        "Derived from https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html#method.worker_total_busy_duration"
    );
    }
}
#[cfg(all(not(tokio_unstable)))]
pub fn start_runtime_monitoring(
    _runtime_handle: tokio::runtime::Handle,
    _tokio_runtime_metric_names: TokioRuntimeMonitorMetricNames,
    _runtime_name: &'static str,
) {
}
