;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $RepeatIterator
    (@struct $RepeatIterator
      (@field $value (@ref $Term)))

    (@derive $size (@get $RepeatIterator))
    (@derive $equals (@get $RepeatIterator))
    (@derive $hash (@get $RepeatIterator))

    (@export $RepeatIterator (@get $RepeatIterator)))

  (export "isRepeatIterator" (func $Term::RepeatIterator::is))
  (export "getRepeatIteratorValue" (func $Term::RepeatIterator::get::value))

  (func $Term::RepeatIterator::startup)

  (func $Term::RepeatIterator::new (export "createRepeatIterator") (param $value i32) (result i32)
    (call $Term::TermType::RepeatIterator::new (local.get $value)))

  (func $Term::RepeatIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $Term::RepeatIterator::get::value (local.get $self))))

  (func $Term::RepeatIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::RepeatIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_value i32)
    (local.set $substituted_value
      (call $Term::traits::substitute
        (call $Term::RepeatIterator::get::value (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_value))
      (then
        (global.get $NULL))
      (else
        (call $Term::RepeatIterator::new (local.get $substituted_value)))))

  (func $Term::RepeatIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::RepeatIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $Term::RepeatIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Term::RepeatIterator::get::value (local.get $self))
    (i32.const 0)
    (global.get $NULL)))
