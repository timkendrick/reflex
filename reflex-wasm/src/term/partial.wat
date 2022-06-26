;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Partial::startup)

  (func $Partial::new (export "createPartial") (param $target i32) (param $args i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Partial) (i32.const 2)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $target))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $args))
    ;; Instantiate the term
    (call $Term::init))

  (func $Partial::is (export "isPartial") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Partial) (call $Term::get_type (local.get $term))))

  (func $Partial::get::target (export "getPartialTarget") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Partial::get::args (export "getPartialArgs") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $Partial::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Partial::traits::is_atomic (param $self i32) (result i32)
    (i32.and
      (call $Term::traits::is_atomic (call $Partial::get::target (local.get $self)))
      (call $Term::traits::is_atomic (call $Partial::get::args (local.get $self)))))

  (func $Partial::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Partial::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Partial::get::target (local.get $self))
    (call $Hash::write_term)
    (call $Partial::get::args (local.get $self))
    (call $Hash::write_term))

  (func $Partial::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.and
      (call $Term::traits::equals (call $Partial::get::target (local.get $self)) (call $Partial::get::target (local.get $other)))
      (call $Term::traits::equals (call $Partial::get::args (local.get $self)) (call $Partial::get::args (local.get $other)))))

  (func $Partial::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Partial::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (call $Term::traits::apply
      (call $Partial::get::target (local.get $self))
      ;; TODO: Convert argument lists to iterators for more efficient partial application
      (call $List::traits::union (call $Partial::get::args (local.get $self)) (local.get $args))
      (local.get $state))))
