;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Symbol::startup)

  (func $Symbol::new (export "createSymbol") (param $id i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Symbol) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $id))
    ;; Instantiate the term
    (call $Term::init))

  (func $Symbol::is (export "isSymbol") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Symbol) (call $Term::get_type (local.get $term))))

  (func $Symbol::get::id (export "getSymbolId") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Symbol::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Symbol::traits::is_atomic (param $self i32) (result i32)
    (call $Symbol::traits::is_static (local.get $self)))

  (func $Symbol::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Symbol::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Symbol::get::id (local.get $self))
    (call $Hash::write_i32))

  (func $Symbol::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.eq (call $Symbol::get::id (local.get $self)) (call $Symbol::get::id (local.get $other))))

  (func $Symbol::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset))))
