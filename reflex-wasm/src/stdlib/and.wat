;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_And "And"
    (@args (@strict $self) (@strict $consequent))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (call $TermType::implements::apply)
      (func $Stdlib_And::impl::Boolean::<apply> (param $self i32) (param $consequent i32) (param $state i32) (result i32 i32)
        ;; Determine whether the condition is true
        (if (result i32 i32)
          (call $Term::Boolean::get::value (local.get $self))
          (then
            ;; If the condition is true, invoke the consequent function with an empty argument list
            (call $Term::traits::apply (local.get $consequent) (call $Term::List::empty) (local.get $state)))
          (else
            ;; Otherwise return the falsy condition
            (local.get $self)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_And::impl::default (param $self i32) (param $consequent i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_And)
            (call $Term::List::create_pair (local.get $self) (local.get $consequent))))
        (global.get $NULL)))))
