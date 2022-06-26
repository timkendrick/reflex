;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_Throw::NAME "name")
  (@const-string $Stdlib_Throw::AGGREGATE_ERROR "AggregateError")
  (@const-string $Stdlib_Throw::ERRORS "errors")

  (@builtin $Stdlib_Throw "Throw"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Throw::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $conditions i32)
        (if (result i32 i32)
          ;; If the thrown error is a (potentially re-thrown) AggregatedError object containing multiple conditions,
          ;; throw a signal that contains all the conditions extracted from within the AggregatedError object
          (i32.ne
            (global.get $NULL)
            (local.tee $conditions (call $Stdlib_Throw::parse_aggregate_error_conditions (local.get $self))))
          (then
            (call $Term::Signal::new (local.get $conditions))
            (global.get $NULL))
          (else
            ;; Otherwise throw a new error signal containing the provided error payload argument
            (call $Term::Signal::of (call $Term::Condition::error (local.get $self)))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Throw::impl::default (param $self i32) (param $state i32) (result i32 i32)
        ;; Wrap the error payload argument into an error condition and return a signal
        (call $Term::Signal::of (call $Term::Condition::error (local.get $self)))
        (global.get $NULL))))

  (func $Stdlib_Throw::parse_aggregate_error_conditions (param $self i32) (result i32)
    (local $error_name i32)
    (local $errors i32)
    (local $conditions i32)
    (local $index i32)
    (if (result i32)
      ;; If the error is not a named JS error object, bail out
      (i32.eq
        (local.tee $error_name (call $Term::Record::traits::get (local.get $self) (global.get $Stdlib_Throw::NAME)))
        (global.get $NULL))
      (then
        (global.get $NULL))
      (else
        ;; If the error is not a JS AggregateError object, bail out
        (if (result i32)
          (i32.eqz (call $Term::traits::equals (local.get $error_name) (global.get $Stdlib_Throw::AGGREGATE_ERROR)))
          (then
            (global.get $NULL))
          (else
            (if (result i32)
              ;; If the AggregateError object does not have an "errors" field, bail out
              (i32.eq
                (local.tee $errors (call $Term::Record::traits::get (local.get $self) (global.get $Stdlib_Throw::ERRORS)))
                (global.get $NULL))
              (then
                (global.get $NULL))
              (else
                (if (result i32)
                  ;; If the AggregateError object "errors" field is not a list, bail out
                  (i32.eqz (call $Term::List::is (local.get $errors)))
                  (then
                    (global.get $NULL))
                  (else
                    (if (result i32)
                      ;; If the AggregateError object "errors" list is empty, bail out
                      (i32.eq
                        (local.tee $index (i32.sub (call $Term::List::get_length (local.get $errors)) (i32.const 1)))
                        (i32.const -1))
                      (then
                        (global.get $NULL))
                      (else
                        ;; Iterate backwards through the list of errors to build up a tree of condition terms
                        (local.set $conditions (global.get $NULL))
                        (loop $LOOP
                          ;; Push the condition onto the accumulated condition tree
                          (local.set $conditions
                            (call $Term::Tree::new
                              ;; Wrap the error payload within an error condition
                              (call $Term::Condition::error
                                (call $Term::List::get_item (local.get $errors) (local.get $index)))
                              (local.get $conditions)))
                          ;; If this is not the final error list item, continue with the next item
                          (i32.ge_u (local.get $index) (i32.const 1))
                          (local.set $index (i32.sub (local.get $index) (i32.const 1)))
                          (br_if $LOOP))
                        ;; Return the accumulated list of conditions
                        (local.get $conditions)))))))))))))
