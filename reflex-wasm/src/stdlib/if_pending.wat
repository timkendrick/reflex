;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_IfPending "IfPending"
    (@args (@eager $self) (@strict $fallback))

    (@impl
      (i32.eq (global.get $TermType::Signal))
      (call $TermType::implements::apply)
      (func $Stdlib_IfPending::impl::Signal::<apply> (param $self i32) (param $fallback i32) (param $state i32) (result i32 i32)
        (local $pending_conditions i32)
        (local $remaining_conditions i32)
        ;; Partition the signal conditions into pending vs non-pending
        (call $Term::Signal::partition_conditions_by_type (local.get $self) (global.get $Condition::PendingCondition))
        (local.set $remaining_conditions)
        (local.set $pending_conditions)
        (if (result i32 i32)
          ;; If the signal does not contain any pending conditions, return the signal as-is
          (i32.eqz (call $Term::List::get_length (local.get $pending_conditions)))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise if all the conditions within the signal were pending conditions, return the fallback value
            (if (result i32 i32)
              (i32.eqz (call $Term::List::get_length (local.get $remaining_conditions)))
              (then
                (call $Term::traits::apply (local.get $fallback) (call $Term::List::empty) (local.get $state)))
              (else
                ;; Otherwise return a signal containing just the non-pending conditions
                (call $Term::Signal::traits::collect
                  (call $Term::List::traits::iterate (local.get $remaining_conditions))
                  (local.get $state))))))))

    (@default
      (func $Stdlib_IfPending::impl::default (param $self i32) (param $fallback i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
