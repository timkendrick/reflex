;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Not "Not"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (func $Stdlib_Not::impl::Boolean (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (i32.eqz (call $Term::Boolean::get::value (local.get $self))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Not::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Not)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
