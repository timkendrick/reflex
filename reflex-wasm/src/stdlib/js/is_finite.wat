;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_IsFinite "IsFinite"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_IsFinite::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (global.get $TRUE))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_IsFinite::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new
          (call $Utils::f64::is_finite (call $Term::Float::get_value (local.get $self))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_IsFinite::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_IsFinite)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
