;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Int::startup)

  (func $Int::new (export "createInt") (param $value i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Int) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value))
    ;; Instantiate the term
    (call $Term::init))

  (func $Int::is (export "isInt") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Int) (call $Term::get_type (local.get $term))))

  (func $Int::get::value (export "getIntValue") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Int::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Int::traits::is_atomic (param $self i32) (result i32)
    (call $Int::traits::is_static (local.get $self)))

  (func $Int::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Int::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Int::get::value (local.get $self))
    (call $Hash::write_i32))

  (func $Int::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.eq (call $Int::get::value (local.get $self)) (call $Int::get::value (local.get $other))))

  (func $Int::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (i32.add
      (local.get $offset)
      (call $Utils::i32::write_string
        (call $Int::get::value (local.get $self))
        (local.get $offset)))))
