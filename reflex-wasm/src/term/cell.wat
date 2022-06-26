;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Cell
    (@struct $Cell
      (@field $fields (@repeated i32)))

    (@derive $size (@get $Cell))

    (@export $Cell (@get $Cell)))

  (export "isCell" (func $Term::Cell::is))

  (func $Cell::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Cells are mutable and therefore cells with different addresses cannot be equal
    (global.get $FALSE))

  (func $Cell::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; Cells are mutable and therefore cells with different addresses must hash to unique values
    (local.get $state)
    (local.get $self)
    (call $Hash::write_i32))

  (func $Term::Cell::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Cell(")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string (call $Term::Cell::get_num_fields (local.get $self)) (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Cell::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the cell fields pointer for an imaginary cell term located
    ;; at memory address 0. The pointer offset tells us how many bytes are taken up by the preceding cell wrapper.
    (call $Term::Cell::get::fields::pointer (i32.const 0) (i32.const 0)))

  (func $Term::Cell::allocate (export "allocateCell") (param $capacity i32) (result i32)
    (local $self i32)
    ;; First allocate a new term wrapper with the required capacity
    (local.tee $self
      (call $Allocator::allocate
        (i32.add
          (call $Term::Cell::empty::sizeof)
          (i32.mul (i32.const 4) (local.get $capacity)))))
    ;; Then manually write the cell struct contents into the term wrapper
    (call $TermType::Cell::construct (call $Term::pointer::value (local.get $self)))
    (call $Term::Cell::set::fields::capacity (local.get $self) (local.get $capacity)))

  (func $Term::Cell::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Cell::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Cell::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Cell::get_num_fields (export "getCellNumFields") (param $self i32) (result i32)
    (call $Term::Cell::get::fields::length (local.get $self)))

  (func $Term::Cell::get_field_pointer (export "getCellFieldPointer") (param $self i32) (param $field i32) (result i32)
    (call $Term::Cell::get::fields::pointer (local.get $self) (local.get $field)))

  (func $Term::Cell::get_field (export "getCellField") (param $self i32) (param $field i32) (result i32)
    (call $Term::Cell::get::fields::value (local.get $self) (local.get $field)))

  (func $Term::Cell::set_field (param $self i32) (param $field i32) (param $value i32)
    (call $Term::Cell::set::fields::value (local.get $self) (local.get $field) (local.get $value))))
