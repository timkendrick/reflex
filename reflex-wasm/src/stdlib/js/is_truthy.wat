;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_IsTruthy "IsTruthy"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Nil))
      (func $Stdlib_IsTruthy::impl::Nil (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (global.get $FALSE))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (func $Stdlib_IsTruthy::impl::Boolean (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_IsTruthy::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new
          (i32.eqz (i64.eqz (call $Term::Int::get_value (local.get $self)))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_IsTruthy::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new
          (i32.eqz (f64.eq (f64.const 0.0) (call $Term::Float::get_value (local.get $self)))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_IsTruthy::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new
          (i32.eqz (i32.eqz (call $Term::String::get::length (local.get $self)))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_IsTruthy::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (global.get $TRUE))
        (global.get $NULL)))))
