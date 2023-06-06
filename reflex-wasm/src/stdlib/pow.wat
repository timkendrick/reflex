;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Pow "Pow"
    (@args (@strict $self) (@strict $exponent))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Pow::impl::Int::Int (param $self i32) (param $exponent i32) (param $state i32) (result i32 i32)
        (local $self_value i64)
        (local $exponent_value i32)
        (local $is_negative_exponent i32)
        (if (result i32 i32)
          ;; If the base is zero and the exponent is negative, return an error
          (i32.and
            (i64.eqz (local.tee $self_value (call $Term::Int::get::value (local.get $self))))
            (local.tee $is_negative_exponent (i32.lt_s (local.tee $exponent_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $exponent)))) (i32.const 0))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Pow)
                (call $Term::List::create_pair (local.get $self) (local.get $exponent))))
            (global.get $NULL))
          (else
            ;; Otherwise perform the exponentiation operation
            (if (result i32 i32)
              (local.get $is_negative_exponent)
              (then
                ;; If the exponent is negative, perform a fast float exponentiation operation
                (call $Term::Float::new
                  (call $Utils::f64::pow_int
                    (f64.convert_i64_s (local.get $self_value))
                    (local.get $exponent_value)))
                (global.get $NULL))
              (else
                ;; If the exponent is non-negative, perform a fast integer exponentiation operation
                (call $Term::Int::new
                  (call $Utils::i64::pow
                    (call $Term::Int::get::value (local.get $self))
                    (local.get $exponent_value)))
                (global.get $NULL)))))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Pow::impl::Float::Float (param $self i32) (param $exponent i32) (param $state i32) (result i32 i32)
        (local $self_value f64)
        (local $exponent_value f64)
        (local $is_integer_exponent i32)
        (local $is_negative_exponent i32)
        (if (result i32 i32)
          ;; If the base is zero and the exponent is negative, or if the base is negative and the exponent is not an integer, return an error
          (i32.or
            (i32.and
              (f64.eq (local.tee $self_value (call $Term::Float::get::value (local.get $self))) (f64.const 0))
              (local.tee $is_negative_exponent (f64.lt (local.tee $exponent_value (call $Term::Float::get::value (local.get $exponent))) (f64.const 0))))
            (i32.and
              (f64.lt (local.get $self_value) (f64.const 0))
              (i32.eqz (local.tee $is_integer_exponent (call $Utils::f64::is_integer (local.get $exponent_value))))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Pow)
                (call $Term::List::create_pair (local.get $self) (local.get $exponent))))
            (global.get $NULL))
          (else
            ;; Otherwise perform the exponentiation operation
            (call $Term::Float::new
              (if (result f64)
                ;; If the exponent is an integer, perform a fast exponentiation operation
                (local.get $is_integer_exponent)
                (then
                  (if (result f64)
                    (i32.and
                      (call $Utils::f64::is_integer (local.get $self_value))
                      (i32.eqz (local.get $is_negative_exponent)))
                    (then
                      ;; If base is an integer and the exponent is a non-negative integer, perform a fast integer exponentiation operation
                      (f64.convert_i64_s
                        (call $Utils::i64::pow
                          (i64.trunc_f64_s (local.get $self_value))
                          (i32.trunc_f64_s (local.get $exponent_value)))))
                    (else
                      ;; If the exponent is an integer, perform a fast float exponentiation operation
                      (call $Utils::f64::pow_int
                        (local.get $self_value)
                        (i32.trunc_f64_s (local.get $exponent_value))))))
                (else
                  ;; Otherwise if the exponent is not an integer, perform the default float exponentiation operation
                  (call $Utils::f64::pow
                    (local.get $self_value)
                    (local.get $exponent_value)))))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Pow::impl::Int::Float (param $self i32) (param $exponent i32) (param $state i32) (result i32 i32)
        (local $self_value i64)
        (local $exponent_value f64)
        (local $is_integer_exponent i32)
        (local $is_negative_exponent i32)
        (if (result i32 i32)
          ;; If the base is zero and the exponent is negative, or if the base is negative and the exponent is not an integer, return an error
          (i32.or
            (i32.and
              (i64.eqz (local.tee $self_value (call $Term::Int::get::value (local.get $self))))
              (local.tee $is_negative_exponent (f64.lt (local.tee $exponent_value (call $Term::Float::get::value (local.get $exponent))) (f64.const 0))))
            (i32.and
              (i64.lt_s (local.get $self_value) (i64.const 0))
              (i32.eqz (local.tee $is_integer_exponent (call $Utils::f64::is_integer (local.get $exponent_value))))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Pow)
                (call $Term::List::create_pair (local.get $self) (local.get $exponent))))
            (global.get $NULL))
          (else
            ;; Otherwise perform the exponentiation operation
            (call $Term::Float::new
              (if (result f64)
                ;; If the exponent is an integer, perform a fast exponentiation operation
                (local.get $is_integer_exponent)
                (then
                  (if (result f64)
                    (local.get $is_negative_exponent)
                    (then
                      ;; If the exponent is negative, perform a fast float exponentiation operation
                      (call $Utils::f64::pow_int
                        (f64.convert_i64_s (local.get $self_value))
                        (i32.trunc_f64_s (local.get $exponent_value))))
                    (else
                      ;; If the exponent is non-negative, perform a fast integer exponentiation operation
                      (f64.convert_i64_s
                        (call $Utils::i64::pow
                          (local.get $self_value)
                          (i32.trunc_f64_s (local.get $exponent_value)))))))
                (else
                  ;; Otherwise if the exponent is not an integer, perform the default float exponentiation operation
                  (call $Utils::f64::pow
                    (f64.convert_i64_s (local.get $self_value))
                    (local.get $exponent_value)))))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Pow::impl::Float::Int (param $self i32) (param $exponent i32) (param $state i32) (result i32 i32)
        (local $self_value f64)
        (local $exponent_value i32)
        (local $is_negative_exponent i32)
        (if (result i32 i32)
          ;; If the base is zero and the exponent is negative, return an error
          (i32.and
            (f64.eq (local.tee $self_value (call $Term::Float::get::value (local.get $self))) (f64.const 0))
            (local.tee $is_negative_exponent (i32.lt_s (local.tee $exponent_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $exponent)))) (i32.const 0))))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Pow)
                (call $Term::List::create_pair (local.get $self) (local.get $exponent))))
            (global.get $NULL))
          (else
            ;; Otherwise perform the exponentiation operation
            (call $Term::Float::new
              (if (result f64)
                (i32.or
                  (i32.eqz (call $Utils::f64::is_integer (local.tee $self_value (call $Term::Float::get::value (local.get $self)))))
                  (local.get $is_negative_exponent))
                (then
                  ;; If the base is not an integer, or if the exponent is negative, perform a fast float exponentiation operation
                  (call $Utils::f64::pow_int
                    (local.get $self_value)
                    (local.get $exponent_value)))
                (else
                  ;; Otherwise if the base is an integer and the exponent is non-negative, perform a fast integer exponentiation operation
                  (f64.convert_i64_s
                    (call $Utils::i64::pow
                      (i64.trunc_f64_s (local.get $self_value))
                      (local.get $exponent_value))))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Pow::impl::default (param $self i32) (param $exponent i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Pow)
            (call $Term::List::create_pair (local.get $self) (local.get $exponent))))
        (global.get $NULL)))))
