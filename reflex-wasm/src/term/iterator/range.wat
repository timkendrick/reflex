;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $RangeIterator::startup)

  (func $RangeIterator::new (export "createRangeIterator") (param $offset i32) (param $length i32) (result i32)
    (local $self i32)
    (if (result i32)
      (i32.eqz (local.get $length))
      (then
        (call $EmptyIterator::new))
      (else
        (local.tee $self (call $Term::new (global.get $TermType::RangeIterator) (i32.const 2)))
        (call $Term::set_field (local.get $self) (i32.const 0) (local.get $offset))
        (call $Term::set_field (local.get $self) (i32.const 1) (local.get $length))
        (call $Term::init))))

  (func $RangeIterator::is (export "isRangeIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::RangeIterator) (call $Term::get_type (local.get $self))))

  (func $RangeIterator::get::offset (export "getRangeIteratorOffset") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $RangeIterator::get::length (export "getRangeIteratorLength") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $RangeIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $RangeIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $RangeIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $RangeIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $RangeIterator::get::offset (local.get $self))
    (call $Hash::write_i32)
    (call $RangeIterator::get::length (local.get $self))
    (call $Hash::write_i32))

  (func $RangeIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (i32.eq
        (call $RangeIterator::get::offset (local.get $self))
        (call $RangeIterator::get::offset (local.get $other)))
      (i32.eq
        (call $RangeIterator::get::length (local.get $self))
        (call $RangeIterator::get::length (local.get $other)))))

  (func $RangeIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $RangeIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $RangeIterator::traits::size_hint (param $self i32) (result i32)
    (call $RangeIterator::get::length (local.get $self)))

  (func $RangeIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; If the given length has been reached, return the complete marker
      (i32.eq
        (local.tee $iterator_state
          ;; Initialize the iterator state at 0
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (call $RangeIterator::get::length (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise return the current value and increment the iterator state
        (call $Int::new
          (i32.add (call $RangeIterator::get::offset (local.get $self)) (local.get $iterator_state)))
        (i32.add (local.get $iterator_state) (i32.const 1))
        (global.get $NULL)))))
