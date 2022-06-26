;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ParseDate "ParseDate"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_ParseDate::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Date::new (i64.extend_i32_s (call $Term::Int::get_value (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_ParseDate::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Date::new (i64.trunc_f64_s (call $Term::Float::get_value (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Date))
      (func $Stdlib_ParseDate::impl::Date (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ParseDate::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local $timestamp i64)
        (if (result i32 i32)
          (i64.ne
            (local.tee $timestamp
              (call $Utils::Date::parse
                (call $Term::String::get_offset (local.get $self))
                (call $Term::String::get_length (local.get $self))))
            (i64.const 0xFFFFFFFFFFFFFFFF))
          (then
            (call $Term::Date::new (local.get $timestamp))
            (global.get $NULL))
          (else
            (call $Stdlib_ParseDate::impl::default (local.get $self) (local.get $state))))))

    (@default
      (func $Stdlib_ParseDate::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ParseDate)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
