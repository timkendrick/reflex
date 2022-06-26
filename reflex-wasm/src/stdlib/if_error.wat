;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_IfError
    (@args (@eager $self) (@strict $handler))

    (@impl
      (i32.eq (global.get $TermType::Signal))
      (call $TermType::implements::apply)
      (func $Stdlib_IfError::impl::Signal::<apply> (param $self i32) (param $handler i32) (param $state i32) (result i32 i32)
        (local $error_conditions i32)
        (local $remaining_conditions i32)
        ;; Partition the signal conditions into error vs non-error
        (call $Signal::partition_conditions_by_type (local.get $self) (global.get $ConditionType::Error))
        (local.set $remaining_conditions)
        (local.set $error_conditions)
        (if (result i32 i32)
          ;; If the signal does not contain any error conditions, return the signal as-is
          (i32.eqz (call $List::get::length (local.get $error_conditions)))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise if all the conditions within the signal were error conditions, invoke the handler
            (if (result i32 i32)
              (i32.eqz (call $List::get::length (local.get $remaining_conditions)))
              (then
                ;; Return the result of applying the handler to a single argument (the list of error conditions)
                (call $Term::traits::apply
                  (local.get $handler)
                  (call $List::of (local.get $error_conditions))
                  (local.get $state)))
              (else
                ;; Otherwise return a signal containing just the non-error conditions
                (call $Signal::traits::collect
                  (call $List::traits::iterate (local.get $remaining_conditions))
                  (local.get $state))))))))

    (@default
      (func $Stdlib_IfError::impl::default (param $self i32) (param $handler i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
