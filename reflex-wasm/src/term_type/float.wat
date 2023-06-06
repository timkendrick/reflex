;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Float
    (@struct $Float
      (@field $value f64))

    (@derive $size (@get $Float))
    (@derive $hash (@get $Float))

    (@export $Float (@get $Float)))

  (export "isFloat" (func $Term::Float::is))

  (func $Float::traits::equals (param $self i32) (param $other i32) (result i32)
    (local $self_value f64)
    (local $other_value f64)
    (i32.or
      ;; Return true either if the two values are identical...
      (f64.eq
        (local.tee $self_value (call $Float::get::value (local.get $self)))
        (local.tee $other_value (call $Float::get::value (local.get $other))))
      ;; ...or both are NaN
      (i32.and
        (call $Utils::f64::is_nan (local.get $self_value))
        (call $Utils::f64::is_nan (local.get $other_value)))))

  (func $Term::Float::new (export "createFloat") (param $value f64) (result i32)
    (call $Term::TermType::Float::new (local.get $value)))

  (func $Term::Float::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Float::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Float::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $Utils::Float::to_string
      (call $Term::Float::get_value (local.get $self))
      (local.get $offset))
    (i32.add (local.get $offset)))

  (func $Term::Float::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Float::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Float::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Float::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $bytes_written i32)
    (if (result i32 i32)
      (i32.eq
        (global.get $NULL)
        (local.tee $bytes_written
          (call $Utils::f64::write_string
            (call $Term::Float::get::value (local.get $self))
            (local.get $offset))))
      (then
        ;; TODO: Decide correct behavior when JSON-serializing NaN or infinite float values
        ;; If the write failed due to NaN or infinite float values, return a null JSON value
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Write the null value to the output string and return the updated offset
        (@store-bytes $offset "null")
        (i32.add (local.get $offset)))
      (else
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Write the serialized value to the output string and return the updated offset
        (i32.add (local.get $offset) (local.get $bytes_written)))))

  (func $Term::Float::get_value (export "getFloatValue") (param $self i32) (result f64)
    (call $Term::Float::get::value (local.get $self)))

  (func $Term::Float::get_non_negative_integer_value (param $self i32) (result i64)
    (local $value f64)
    (select
      (i64.trunc_f64_s (local.tee $value (call $Term::Float::get::value (local.get $self))))
      (i64.const 0xFFFFFFFFFFFFFFFF)
      (i32.and
        (call $Utils::f64::is_integer (local.get $value))
        (f64.ge (local.get $value) (f64.const 0))))))
