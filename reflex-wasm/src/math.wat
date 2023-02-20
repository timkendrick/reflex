;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Imported floating-point Math functions
  (func $Utils::f64::remainder (import "Math" "remainder") (param f64 f64) (result f64))
  (func $Utils::f64::acos (import "Math" "acos") (param f64) (result f64))
  (func $Utils::f64::acosh (import "Math" "acosh") (param f64) (result f64))
  (func $Utils::f64::asin (import "Math" "asin") (param f64) (result f64))
  (func $Utils::f64::asinh (import "Math" "asinh") (param f64) (result f64))
  (func $Utils::f64::atan (import "Math" "atan") (param f64) (result f64))
  (func $Utils::f64::atan2 (import "Math" "atan2") (param f64 f64) (result f64))
  (func $Utils::f64::atanh (import "Math" "atanh") (param f64) (result f64))
  (func $Utils::f64::cbrt (import "Math" "cbrt") (param f64) (result f64))
  (func $Utils::f64::cos (import "Math" "cos") (param f64) (result f64))
  (func $Utils::f64::cosh (import "Math" "cosh") (param f64) (result f64))
  (func $Utils::f64::exp (import "Math" "exp") (param f64) (result f64))
  (func $Utils::f64::expm1 (import "Math" "expm1") (param f64) (result f64))
  (func $Utils::f64::hypot (import "Math" "hypot") (param f64 f64) (result f64))
  (func $Utils::f64::log (import "Math" "log") (param f64) (result f64))
  (func $Utils::f64::log2 (import "Math" "log2") (param f64) (result f64))
  (func $Utils::f64::log10 (import "Math" "log10") (param f64) (result f64))
  (func $Utils::f64::log1p (import "Math" "log1p") (param f64) (result f64))
  (func $Utils::f64::pow (import "Math" "pow") (param f64 f64) (result f64))
  (func $Utils::f64::sin (import "Math" "sin") (param f64) (result f64))
  (func $Utils::f64::sinh (import "Math" "sinh") (param f64) (result f64))
  (func $Utils::f64::sqrt (import "Math" "sqrt") (param f64) (result f64))
  (func $Utils::f64::tan (import "Math" "tan") (param f64) (result f64))
  (func $Utils::f64::tanh (import "Math" "tanh") (param f64) (result f64)))
