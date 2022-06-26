;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Cell::startup)

  (func $Cell::new (param $num_fields i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Cell) (local.get $num_fields)))
    ;; Instantiate the term
    (call $Term::init))

  (func $Cell::is (export "isCell") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Cell) (call $Term::get_type (local.get $term))))

  (func $Cell::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Cell::traits::is_atomic (param $self i32) (result i32)
    (call $Cell::traits::is_static (local.get $self)))

  (func $Cell::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Cell::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the cell pointer
    ;; (cells are mutable and therefore cells with different addresses must hash to unique values)
    (local.get $self)
    (call $Hash::write_i32))

  (func $Cell::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Cells are mutable and therefore cells with different addresses cannot be equal
    (global.get $FALSE))

  (func $Cell::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Cell::get_num_fields (export "getCellNumFields") (param $self i32) (result i32)
    (call $Term::get_num_fields (local.get $self)))

  (func $Cell::get_field_pointer (export "getCellFieldPointer") (param $self i32) (param $field i32) (result i32)
    (call $Term::get_field_pointer (local.get $self) (local.get $field)))

  (func $Cell::get_field (export "getCellField") (param $self i32) (param $field i32) (result i32)
    (call $Term::get_field (local.get $self) (local.get $field)))

  (func $Cell::set_field (param $self i32) (param $field i32) (param $value i32)
    (call $Term::set_field (local.get $self) (local.get $field) (local.get $value))))
