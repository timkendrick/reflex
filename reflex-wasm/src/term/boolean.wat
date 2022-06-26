;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Boolean::TRUE (mut i32) (i32.const -1))
  (global $Boolean::FALSE (mut i32) (i32.const -1))

  (func $Boolean::startup
    ;; Pre-allocate the singleton instances
    (local $instance i32)
    ;; Allocate a new struct of the required size and type (one field for the value; no additional bytes)
    (local.tee $instance (call $Term::new (global.get $TermType::Boolean) (i32.const 1)))
    ;; Store the value as the first field
    (call $Term::set_field (local.get $instance) (i32.const 0) (i32.const 1))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $Boolean::TRUE)
    ;; Allocate a new struct of the required size and type (one field for the value; no additional bytes)
    (local.tee $instance (call $Term::new (global.get $TermType::Boolean) (i32.const 1)))
    ;; Store the value as the first field
    (call $Term::set_field (local.get $instance) (i32.const 0) (i32.const 0))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $Boolean::FALSE))

  (func $Boolean::new (export "createBoolean") (param $value i32) (result i32)
    ;; Return the pre-allocated singleton instance
    (if (result i32)
      (local.get $value)
      (then
        (call $Boolean::true))
      (else
        (call $Boolean::false))))

  (func $Boolean::true (result i32)
    (global.get $Boolean::TRUE))

  (func $Boolean::false (result i32)
    (global.get $Boolean::FALSE))

  (func $Boolean::is (export "isBoolean") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Boolean) (call $Term::get_type (local.get $term))))

  (func $Boolean::get::value (export "getBooleanValue") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Boolean::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Boolean::traits::is_atomic (param $self i32) (result i32)
    (call $Boolean::traits::is_static (local.get $self)))

  (func $Boolean::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Boolean::get::value (local.get $self))
    (call $Hash::write_i32))

  (func $Boolean::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.eq (call $Boolean::get::value (local.get $self)) (call $Boolean::get::value (local.get $other))))

  (func $Boolean::traits::is_truthy (param $self i32) (result i32)
    (call $Boolean::get::value (local.get $self)))

  (func $Boolean::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (if (result i32)
      (call $Boolean::get::value (local.get $self))
      (then
        (@store_bytes (local.get $offset) "true")
        (i32.add (local.get $offset)))
      (else
        (@store_bytes (local.get $offset) "false")
        (i32.add (local.get $offset))))))
