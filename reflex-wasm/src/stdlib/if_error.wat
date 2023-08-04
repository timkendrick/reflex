;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_IfError "IfError"
    (@args (@eager $self) (@strict $handler))

    (@impl
      (i32.eq (global.get $TermType::Signal))
      (call $TermType::implements::apply)
      (func $Stdlib_IfError::impl::Signal::<apply> (param $self i32) (param $handler i32) (param $state i32) (result i32 i32)
        (local $error_conditions i32)
        (local $remaining_conditions i32)
        (local $num_error_conditions i32)
        (local $error_payloads i32)
        (local $index i32)
        ;; Partition the signal conditions into error vs non-error
        (call $Term::Signal::partition_conditions_by_type (local.get $self) (global.get $Condition::ErrorCondition))
        (local.set $remaining_conditions)
        (local.set $error_conditions)
        (if (result i32 i32)
          ;; If the signal does not contain any error conditions, return the signal as-is
          (i32.eqz (local.tee $num_error_conditions (call $Term::List::get_length (local.get $error_conditions))))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise if all the conditions within the signal were error conditions, invoke the handler
            (if (result i32 i32)
              (i32.eqz (call $Term::List::get_length (local.get $remaining_conditions)))
              (then
                ;; Collect the error condition payloads into a list
                ;; TODO: Avoid unnecessary heap allocations for intermediate values
                ;; Initialize a new list to store the error payloads
                (local.set $error_payloads (call $Term::List::allocate (local.get $num_error_conditions)))
                ;; Iterate over the error conditions, extracting the payload from each and adding it to the list
                (loop $LOOP
                  (call $Term::List::set_item
                    (local.get $error_payloads)
                    (local.get $index)
                    ;; Extract the payload from the current error condition
                    (call $Term::Condition::ErrorCondition::get::payload
                      (call $Term::List::get_item (local.get $error_conditions) (local.get $index))))
                  ;; If there are more conditions remaining, continue with the next condition
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                      (local.get $num_error_conditions))))
                ;; Initiate the newly-allocated list
                (local.set $error_payloads
                  (call $Term::List::init (local.get $error_payloads) (local.get $num_error_conditions)))
                ;; Return the result of applying the handler to a single argument (the list of error condition payloads)
                (call $Term::traits::apply
                  (local.get $handler)
                  (call $Term::List::of (local.get $error_payloads))
                  (local.get $state)))
              (else
                ;; Otherwise return a signal containing just the non-error conditions
                (call $Term::Signal::traits::collect
                  (call $Term::List::traits::iterate (local.get $remaining_conditions))
                  (local.get $state))))))))

    (@impl
      (i32.or (i32.const 0xFFFFFFFF))
      (call $TermType::implements::apply)
      (func $Stdlib_IfError::impl::any::<apply> (param $self i32) (param $fallback i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@default
      (func $Stdlib_IfError::impl::default (param $self i32) (param $handler i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_IfError)
            (call $Term::List::create_pair (local.get $self) (local.get $handler))))
        (global.get $NULL)))))
