// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export function createTestRunner(setup, { assert, hrtime, print }) {
  function bench(label, task, testFn) {
    if (typeof console === 'object' && typeof console.profile === 'function')
      console.profile(label);
    const startLabel = `${label} [start]`;
    const endLabel = `${label} [end]`;
    const startMark =
      typeof performance === 'object' && typeof performance.mark === 'function'
        ? performance.mark(startLabel)
        : undefined;
    const startTime = hrtime.bigint();
    const result = task();
    const endTime = hrtime.bigint();
    const endMark =
      typeof performance === 'object' && typeof performance.mark === 'function'
        ? performance.mark(endLabel)
        : undefined;
    if (typeof console === 'object' && typeof console.profile === 'function')
      console.profileEnd(label);
    if (startMark && endMark) performance.measure(label, startLabel, endLabel);
    print(`${formatNanoseconds(Number(endTime - startTime))} `);
    if (testFn) testFn(result, assert);
  }
  function formatNanoseconds(value) {
    const units = ['ns', 'µs', 'ms', 's'];
    while (value >= 1000 && units.length > 1) {
      value /= 1000;
      units.shift();
    }
    return `${value.toFixed(2)}${units[0]}`;
  }
  return (factory) => {
    const suites = [];
    factory((label, suiteFactory) => {
      const tests = [];
      function skippable(createTest) {
        return Object.assign(
          function test(label, fn) {
            tests.push({ ...createTest(label, fn), skip: false, skipOthers: false });
          },
          {
            skip: (label, fn) => {
              tests.push({ ...createTest(label, fn), skip: true, skipOthers: false });
            },
            only: (label, fn) => {
              tests.push({ ...createTest(label, fn), skip: false, skipOthers: true });
            },
          },
        );
      }
      suiteFactory(
        skippable(function createTest(label, testFn) {
          return {
            label,
            fn: () => setup().then((value) => testFn(assert, value)),
          };
        }),
        skippable(function createBenchmark(label, benchFactory) {
          return {
            label,
            fn: () =>
              setup().then((value) => {
                benchFactory(bench.bind(null, label), value);
              }),
          };
        }),
      );
      suites.push({
        label,
        tests,
      });
    }, assert);
    const startTime = Date.now();
    const tests = suites.flatMap((suite) => suite.tests);
    const only = tests.filter(({ skipOthers }) => skipOthers);
    return clearTerminal(print)
      .then(() => {
        return suites.reduce(
          (previous, suite) =>
            previous.then((combinedResults) => {
              print(`\n\n${bold(suite.label)} (${suite.tests.length} tests)\n\n`);
              return suite.tests
                .reduce(
                  (previous, test) =>
                    previous.then((results) => {
                      const shouldSkip = test.skip || (only.length > 0 && !only.includes(test));
                      print(`${test.label}: `);
                      if (shouldSkip) {
                        print(`${yellow('SKIP')}\n`);
                        results.push({ type: 'skip' });
                        return results;
                      } else {
                        return test
                          .fn()
                          .then(() => {
                            print(`${green('PASS')}\n`);
                            results.push({ type: 'pass' });
                            return results;
                          })
                          .catch((err) => {
                            print(`${red('FAIL')}\n`);
                            results.push({
                              type: 'fail',
                              label: `${suite.label} ▸ ${test.label}`,
                              error: err,
                            });
                            return results;
                          });
                      }
                    }),
                  Promise.resolve([]),
                )
                .then((results) => {
                  combinedResults.push(...results);
                  return combinedResults;
                });
            }),
          Promise.resolve([]),
        );
      })
      .then((results) => {
        const elapsedTime = Date.now() - startTime;
        const failures = results.filter(({ type }) => type === 'fail');
        const numPassed = results.filter(({ type }) => type === 'pass').length;
        const numSkipped = results.filter(({ type }) => type === 'skip').length;
        const numFailed = failures.length;
        print(
          `\n---\n\n${
            results.length === 0
              ? red('No tests run')
              : [
                  numPassed > 0
                    ? `${numPassed} ${numPassed === 1 ? 'test' : 'tests'} passed`
                    : null,
                  numSkipped > 0
                    ? yellow(`${numSkipped} ${numSkipped === 1 ? 'test' : 'tests'} skipped`)
                    : null,
                  numFailed > 0
                    ? red(`${numFailed} ${numFailed === 1 ? 'test' : 'tests'} failed`)
                    : null,
                ]
                  .filter(Boolean)
                  .join(', ')
          } (${formatTime(elapsedTime)})\n\n`,
        );
        if (numFailed > 0) {
          failures.forEach(({ label, error }, index) => {
            print(`${index === 0 ? '' : '\n'}${red(`${label}:`)}\n\n`);
            console.error(error);
          });
          return false;
        } else {
          return true;
        }
      });
  };
}

function formatTime(duration) {
  return `${Math.floor(duration)}ms`;
}

function clearTerminal(print) {
  print(clear());
  return new Promise((resolve) => setTimeout(resolve, 30));
}

function clear() {
  return '\u001b[2J\u001b[3J\u001b[H';
}

function bold(label) {
  return `\u001b[1m${label}\u001b[0m`;
}

function red(label) {
  return `\u001b[31m${label}\u001b[0m`;
}

function green(label) {
  return `\u001b[32m${label}\u001b[0m`;
}

function yellow(label) {
  return `\u001b[33m${label}\u001b[0m`;
}
