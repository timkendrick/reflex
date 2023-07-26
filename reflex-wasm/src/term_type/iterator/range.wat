;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $RangeIterator
    (@struct $RangeIterator
      (@field $offset i64)
      (@field $length i32))

    (@derive $size (@get $RangeIterator))
    (@derive $equals (@get $RangeIterator))
    (@derive $hash (@get $RangeIterator))

    (@export $RangeIterator (@get $RangeIterator)))

  (export "isRangeIterator" (func $Term::RangeIterator::is))
  (export "getRangeIteratorOffset" (func $Term::RangeIterator::get::offset))
  (export "getRangeIteratorLength" (func $Term::RangeIterator::get::length))

  (func $Term::RangeIterator::new (export "createRangeIterator") (param $offset i64) (param $length i32) (result i32)
    (if (result i32)
      (i32.eqz (local.get $length))
      (then
        (call $Term::EmptyIterator::new))
      (else
        (call $Term::TermType::RangeIterator::new (local.get $offset) (local.get $length)))))

  (func $Term::RangeIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::RangeIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::RangeIterator) (local.get $offset)))

  (func $Term::RangeIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::RangeIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::RangeIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::RangeIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::RangeIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::RangeIterator::get::length (local.get $self)))

  (func $Term::RangeIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; If the given length has been reached, return the complete marker
      (i32.eq
        (local.tee $iterator_state
          ;; Initialize the iterator state at 0
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (call $Term::RangeIterator::get::length (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise return the current value and increment the iterator state
        (call $Term::Int::new
          (i64.add
            (call $Term::RangeIterator::get::offset (local.get $self))
            (i64.extend_i32_u (local.get $iterator_state))))
        (i32.add (local.get $iterator_state) (i32.const 1))
        (global.get $NULL)))))
