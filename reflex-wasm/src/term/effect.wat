;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Effect
    (@struct $Effect
      (@field $condition (@ref $Term)))

    (@derive $size (@get $Effect))
    (@derive $equals (@get $Effect))
    (@derive $hash (@get $Effect))

    (@export $Effect (@get $Effect)))

  (export "isEffect" (func $Term::Effect::is))
  (export "getEffectCondition" (func $Term::Effect::get::condition))

  (func $Term::Effect::new (export "createEffect") (param $condition i32) (result i32)
    (call $Term::TermType::Effect::new (local.get $condition)))

  (func $Term::Effect::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Effect::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Effect::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "<!")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Effect::get::condition (local.get $self))
        (local.get $offset)))
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::Effect::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Effect::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Effect::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Effect::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $condition i32)
    (local $value i32)
    (local $dependencies i32)
    (call $Runtime::get_state_value
      (local.tee $condition (call $Term::Effect::get::condition (local.get $self)))
      (local.get $state))
    (local.set $dependencies)
    (local.set $value)
    (if (result i32 i32)
      (i32.eq (global.get $NULL) (local.get $value))
      (then
        (call $Term::Signal::of (local.get $condition))
        (local.get $dependencies))
      (else
        (local.get $value)
        (local.get $dependencies)))))
