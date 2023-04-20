;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ParseInt "ParseInt"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_ParseInt::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_ParseInt::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (local $value f64)
        (if (result i32 i32)
          (i32.and
            (call $Utils::f64::is_finite (local.tee $value (call $Term::Float::get_value (local.get $self))))
            (i32.and
              (f64.ge (local.get $value) (f64.convert_i32_s (i32.const -0x7FFFFFFF)))
              (f64.le (local.get $value) (f64.convert_i32_s (i32.const 0x7FFFFFFF)))))
          (then
            (call $Term::Int::new (i64.trunc_f64_s (local.get $value)))
            (global.get $NULL))
          (else
            (call $Stdlib_ParseInt::impl::default (local.get $self) (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::Date))
      (func $Stdlib_ParseInt::impl::Date (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (call $Term::Date::get::timestamp (local.get $self)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_ParseInt::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ParseInt)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
