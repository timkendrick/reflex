;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Nil::INSTANCE (mut i32) (i32.const -1))

  (func $Nil::startup
    ;; Pre-allocate the singleton instance
    (call $Term::new (global.get $TermType::Nil) (i32.const 0))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $Nil::INSTANCE))

  (func $Nil::new (export "createNil") (result i32)
    (local $self i32)
    ;; Return the pre-allocated singleton instance
    (global.get $Nil::INSTANCE))

  (func $Nil::is (export "isNil") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Nil) (call $Term::get_type (local.get $term))))

  (func $Nil::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Nil::traits::is_atomic (param $self i32) (result i32)
    (call $Nil::traits::is_static (local.get $self)))

  (func $Nil::traits::is_truthy (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Nil::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; All null values are identical, so no need to add anything to the hash state
    (local.get $state))

  (func $Nil::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; All null values are identical, so by definition any two null values must be equal
    (global.get $TRUE))

  (func $Nil::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (@store_bytes (local.get $offset) "null")
    (i32.add (local.get $offset))))
