;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Pointer
    (@struct $Pointer
      (@field $target (@ref $Term)))

    (@derive $size (@get $Pointer))
    (@derive $equals (@get $Pointer))
    (@derive $hash (@get $Pointer))

    (@export $Pointer (@get $Pointer)))

  (export "isPointer" (func $Term::Pointer::is))
  (export "getPointerTarget" (func $Term::Pointer::get::target))

  (func $Term::Pointer::new (export "createPointer") (param $target i32) (result i32)
    (call $Term::TermType::Pointer::new (local.get $target)))

  (func $Term::Pointer::traits::is_atomic (param $self i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::is_atomic (call $Term::Pointer::dereference (local.get $self))))

  (func $Term::Pointer::traits::is_truthy (param $self i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::is_truthy (call $Term::Pointer::dereference (local.get $self))))

  (func $Term::Pointer::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Pointer(")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset
      (call $Term::traits::display
        (call $Term::Pointer::get::target (local.get $self)) (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Pointer::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Invoke the method on the target term
    (call $Term::traits::to_json (call $Term::Pointer::dereference (local.get $self)) (local.get $offset)))

  (func $Term::Pointer::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    ;; Invoke the method on the target term
    (call $Term::traits::substitute (call $Term::Pointer::dereference (local.get $self)) (local.get $variables) (local.get $scope_offset)))

  (func $Term::Pointer::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    ;; Invoke the method on the target term
    (call $Term::traits::evaluate (call $Term::Pointer::dereference (local.get $self)) (local.get $state)))

  (func $Term::Pointer::dereference (param $self i32) (result i32)
    (loop $LOOP
      (br_if $LOOP (call $Term::Pointer::is (local.tee $self (call $Term::Pointer::get::target (local.get $self))))))
    (local.get $self)))
