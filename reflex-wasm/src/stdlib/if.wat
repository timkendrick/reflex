;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_If "If"
    (@args (@strict $self) (@strict $consequent) (@strict $alternative))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (call $TermType::implements::apply)
      (call $TermType::implements::apply)
      (func $Stdlib_If::impl::Boolean::<apply>::<apply> (param $self i32) (param $consequent i32) (param $alternative i32) (param $state i32) (result i32 i32)
        ;; Select the consequent or alternative branch, depending on whether the condition is true
        (select
          (local.get $consequent)
          (local.get $alternative)
          (call $Term::Boolean::get::value (local.get $self)))
        ;; Invoke the branch function with an empty argument list
        (call $Term::traits::apply (call $Term::List::empty) (local.get $state))))

    (@default
      (func $Stdlib_If::impl::default (param $self i32) (param $consequent i32) (param $alternative i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_If)
            (call $Term::List::create_triple (local.get $self) (local.get $consequent) (local.get $alternative))))
        (global.get $NULL)))))
