;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Divide "Divide"
    (@args (@strict $self) (@strict $divisor))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Divide::impl::Int::Int (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value i64)
        (if (result i32 i32)
          (i64.eqz (local.tee $divisor_value (call $Term::Int::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Divide)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Int::new (i64.div_s (call $Term::Int::get::value (local.get $self)) (local.get $divisor_value)))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Divide::impl::Float::Float (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value f64)
        (if (result i32 i32)
          (f64.eq (f64.const 0) (local.tee $divisor_value (call $Term::Float::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Divide)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new (f64.div (call $Term::Float::get::value (local.get $self)) (local.get $divisor_value)))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Divide::impl::Int::Float (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value f64)
        (if (result i32 i32)
          (f64.eq (f64.const 0) (local.tee $divisor_value (call $Term::Float::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Divide)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new (f64.div (f64.convert_i64_s (call $Term::Int::get::value (local.get $self))) (local.get $divisor_value)))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Divide::impl::Float::Int (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value i64)
        (if (result i32 i32)
          (i64.eqz (local.tee $divisor_value (call $Term::Int::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Divide)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new (f64.div (call $Term::Float::get::value (local.get $self)) (f64.convert_i64_s (local.get $divisor_value))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Divide::impl::default (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Divide)
            (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
        (global.get $NULL)))))
