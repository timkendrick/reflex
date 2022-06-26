;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Pointer::startup)

  (func $Pointer::new (export "createPointer") (param $target i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Pointer) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $target))
    ;; Instantiate the term
    (call $Term::init))

  (func $Pointer::is (export "isPointer") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Pointer) (call $Term::get_type (local.get $term))))

  (func $Pointer::traits::is_static (param $self i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::is_static (call $Pointer::get::target (local.get $self))))

  (func $Pointer::traits::is_atomic (param $self i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::is_atomic (call $Pointer::get::target (local.get $self))))

  (func $Pointer::traits::is_truthy (param $self i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::is_truthy (call $Pointer::get::target (local.get $self))))

  (func $Pointer::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::hash (call $Pointer::get::target (local.get $self)) (local.get $state)))

  (func $Pointer::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::equals (call $Pointer::get::target (local.get $self)) (local.get $other)))

  (func $Pointer::traits::write_json (param $self i32) (param $offset i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::write_json (call $Pointer::get::target (local.get $self)) (local.get $offset)))

  (func $Pointer::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    ;; Invoke the method on the target term
    (call $Term::traits::evaluate (call $Pointer::get::target (local.get $self)) (local.get $state)))

  (func $Pointer::get::target (export "getPointerTarget") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Pointer::set_target (param $self i32) (param $value i32)
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value)))

  (func $Pointer::dereference (param $self i32) (result i32)
    (loop $LOOP
      (br_if $LOOP (call $Pointer::is (local.tee $self (call $Pointer::get::target (local.get $self))))))
    (local.get $self)))
