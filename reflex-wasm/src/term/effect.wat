;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Effect::startup)

  (func $Effect::new (export "createEffect") (param $condition i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Effect) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $condition))
    ;; Instantiate the term
    (call $Term::init))

  (func $Effect::is (export "isEffect") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Effect) (call $Term::get_type (local.get $term))))

  (func $Effect::get::condition (export "getEffectCondition") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Effect::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Effect::get::condition (local.get $self))
    (call $Hash::write_term))

  (func $Effect::traits::is_static (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Effect::traits::is_atomic (param $self i32) (result i32)
    (call $Effect::traits::is_static (local.get $self)))

  (func $Effect::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Effect::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (call $Term::traits::equals
      (call $Effect::get::condition (local.get $self))
      (call $Effect::get::condition (local.get $other))))

  (func $Effect::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Effect::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $condition i32)
    (local $value i32)
    (local $dependencies i32)
    (call $Runtime::get_state_value
      (local.tee $condition (call $Effect::get::condition (local.get $self)))
      (local.get $state))
    (local.set $dependencies)
    (local.set $value)
    (if (result i32 i32)
      (i32.eq (global.get $NULL) (local.get $value))
      (then
        (call $Signal::of (local.get $condition))
        (local.get $dependencies))
      (else
        (local.get $value)
        (local.get $dependencies)))))
