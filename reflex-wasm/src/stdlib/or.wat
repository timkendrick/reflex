;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Or "Or"
    (@args (@strict $self) (@strict $alternative))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (call $TermType::implements::apply)
      (func $Stdlib_Or::impl::Boolean::<apply> (param $self i32) (param $alternative i32) (param $state i32) (result i32 i32)
        ;; Determine whether the condition is true
        (if (result i32 i32)
          (call $Term::Boolean::get::value (local.get $self))
          (then
            ;; If the condition is true, return the condition
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise invoke the alternative function with an empty argument list
            (call $Term::traits::apply (local.get $alternative) (call $Term::List::empty) (local.get $state))))))

    (@default
      (func $Stdlib_Or::impl::default (param $self i32) (param $alternative i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Or)
            (call $Term::List::create_pair (local.get $self) (local.get $alternative))))
        (global.get $NULL)))))
