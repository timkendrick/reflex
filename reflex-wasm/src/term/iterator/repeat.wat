;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $RepeatIterator::startup)

  (func $RepeatIterator::new (export "createRepeatIterator") (param $value i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::RepeatIterator) (i32.const 1)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value))
    (call $Term::init))

  (func $RepeatIterator::is (export "isRepeatIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::RepeatIterator) (call $Term::get_type (local.get $self))))

  (func $RepeatIterator::get::value (export "getRepeatIteratorValue") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $RepeatIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $RepeatIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $RepeatIterator::get::value (local.get $self))))

  (func $RepeatIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $RepeatIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $RepeatIterator::get::value (local.get $self))
    (call $Hash::write_term))

  (func $RepeatIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $RepeatIterator::get::value (local.get $self))
      (call $RepeatIterator::get::value (local.get $other))))

  (func $RepeatIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $RepeatIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $RepeatIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $RepeatIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $RepeatIterator::get::value (local.get $self))
    (i32.const 0)
    (global.get $NULL)))
