;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ConstructRecord "ConstructRecord"
    (@args (@strict $keys) (@strict $values))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::iterate)
      (func $Stdlib_ConstructRecord::impl::<iterate>::<iterate> (param $keys i32) (param $values i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; Collect the keys into a list
        (call $Term::List::traits::collect (local.get $keys) (local.get $state))
        ;; Update the accumulated dependencies
        (local.set $dependencies)
        ;; Pop the list of keys off the stack and store as a local variable
        (local.set $keys)
        ;; Collect the values into a list
        (call $Term::List::traits::collect (local.get $values) (local.get $state))
        ;; Update the accumulated dependencies, leaving the list of values on the stack
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; Pop the list of values off the stack and store as a local variable
        (local.set $values)
        ;; If the key and value lists differ in length, return an error
        (if (result i32 i32)
          (i32.ne
            (call $Term::List::get_length (local.get $keys))
            (call $Term::List::get_length (local.get $values)))
          (then
            (call $Stdlib_ConstructRecord::impl::default (local.get $keys) (local.get $values) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          (else
            ;; Otherwise instantiate a new record term composed of the key and value lists
            (call $Term::Record::new (local.get $keys) (local.get $values))
            (local.get $dependencies)))))

    (@default
      (func $Stdlib_ConstructRecord::impl::default (param $keys i32) (param $values i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ConstructRecord)
            (call $Term::List::create_pair (local.get $keys) (local.get $values))))
        (global.get $NULL)))))
