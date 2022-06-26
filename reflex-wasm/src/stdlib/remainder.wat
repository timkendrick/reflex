;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Remainder "Remainder"
    (@args (@strict $self) (@strict $divisor))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Remainder::impl::Int::Int (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value i32)
        (if (result i32 i32)
          (i32.eqz (local.tee $divisor_value (call $Term::Int::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Remainder)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Int::new (i32.rem_s (call $Term::Int::get::value (local.get $self)) (local.get $divisor_value)))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Remainder::impl::Float::Float (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value f64)
        (if (result i32 i32)
          (f64.eq (f64.const 0) (local.tee $divisor_value (call $Term::Float::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Remainder)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new
              (if (result f64)
                (call $Utils::f64::is_integer (local.get $divisor_value))
                (then
                  ;; If the divisor is an integer, perform a fast remainder operation
                  (call $Utils::f64::remainder_int
                    (call $Term::Float::get::value (local.get $self))
                    (i32.trunc_f64_s (local.get $divisor_value))))
                (else
                  ;; Otherwise fall back to the default remainder operation
                  (call $Utils::f64::remainder
                    (call $Term::Float::get::value (local.get $self))
                    (local.get $divisor_value)))))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Remainder::impl::Int::Float (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $divisor_value f64)
        (if (result i32 i32)
          (f64.eq (f64.const 0) (local.tee $divisor_value (call $Term::Float::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Remainder)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new
              (if (result f64)
                (call $Utils::f64::is_integer (local.get $divisor_value))
                (then
                  ;; If the divisor is an integer, perform a fast integer remainder operation
                  (f64.convert_i32_s
                    (i32.rem_s
                      (call $Term::Int::get::value (local.get $self))
                      (i32.trunc_f64_s (local.get $divisor_value)))))
                (else
                  ;; Otherwise fall back to the default float remainder operation
                  (call $Utils::f64::remainder
                    (f64.convert_i32_s (call $Term::Int::get::value (local.get $self)))
                    (local.get $divisor_value)))))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Remainder::impl::Float::Int (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (local $self_value f64)
        (local $divisor_value i32)
        (if (result i32 i32)
          (i32.eqz (local.tee $divisor_value (call $Term::Int::get::value (local.get $divisor))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Remainder)
                (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
            (global.get $NULL))
          (else
            (call $Term::Float::new
              (if (result f64)
                (call $Utils::f64::is_integer (local.tee $self_value (call $Term::Float::get::value (local.get $self))))
                (then
                  ;; If the base is an integer, perform a fast integer remainder operation
                  (f64.convert_i32_s
                    (i32.rem_s
                      (i32.trunc_f64_s (local.get $self_value))
                      (local.get $divisor_value))))
                (else
                  ;; Otherwise fall perform a fast float remainder operation
                  (call $Utils::f64::remainder_int
                    (local.get $self_value)
                    (local.get $divisor_value)))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Remainder::impl::default (param $self i32) (param $divisor i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Remainder)
            (call $Term::List::create_pair (local.get $self) (local.get $divisor))))
        (global.get $NULL)))))
