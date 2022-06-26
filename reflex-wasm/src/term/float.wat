;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Float::startup)

  (func $Float::new (export "createFloat") (param $value f64) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Float) (i32.const 2)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_f64_field (local.get $self) (i32.const 0) (local.get $value))
    ;; Instantiate the term
    (call $Term::init))

  (func $Float::is (export "isFloat") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Float) (call $Term::get_type (local.get $term))))

  (func $Float::get::value (export "getFloatValue") (param $self i32) (result f64)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_f64_field (local.get $self) (i32.const 0)))

  (func $Float::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Float::traits::is_atomic (param $self i32) (result i32)
    (call $Float::traits::is_static (local.get $self)))

  (func $Float::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Float::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Float::get::value (local.get $self))
    (call $Hash::write_f64))

  (func $Float::traits::equals (param $self i32) (param $other i32) (result i32)
    (local $self_value f64)
    (local $other_value f64)
    (i32.or
      ;; Return true if either the two values are identical
      (f64.eq
        (local.tee $self_value (call $Float::get::value (local.get $self)))
        (local.tee $other_value (call $Float::get::value (local.get $other))))
      ;; Or both are NaN
      (i32.and
        (call $Utils::f64::is_nan (local.get $self_value))
        (call $Utils::f64::is_nan (local.get $other_value)))))

  (func $Float::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (local $bytes_written i32)
    (if (result i32)
      (i32.eq
        (global.get $NULL)
        (local.tee $bytes_written
          (call $Utils::f64::write_string
            (call $Float::get::value (local.get $self))
            (local.get $offset))))
      (then
        ;; If the write failed due to NaN or infinite float values, return a null JSON value
        (@store_bytes (local.get $offset) "null")
        (i32.add (local.get $offset)))
      (else
        (i32.add (local.get $offset) (local.get $bytes_written)))))

  (func $Float::get_non_negative_integer_value (param $self i32) (result i32)
    (local $value f64)
    (select
      (i32.trunc_f64_s (local.tee $value (call $Float::get::value (local.get $self))))
      (global.get $NULL)
      (i32.and
        (call $Utils::f64::is_integer (local.get $value))
        (f64.ge (local.get $value) (f64.const 0))))))
