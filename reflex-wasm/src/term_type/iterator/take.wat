;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $TakeIterator
    (@struct $TakeIterator
      (@field $source (@ref $Term))
      (@field $count i32))

    (@derive $size (@get $TakeIterator))
    (@derive $equals (@get $TakeIterator))
    (@derive $hash (@get $TakeIterator))

    (@export $TakeIterator (@get $TakeIterator)))

  (export "isTakeIterator" (func $Term::TakeIterator::is))
  (export "getTakeIteratorSource" (func $Term::TakeIterator::get::source))
  (export "getTakeIteratorCount" (func $Term::TakeIterator::get::count))

  (func $Term::TakeIterator::new (export "createTakeIterator") (param $source i32) (param $count i32) (result i32)
    (if (result i32)
      (i32.eqz (local.get $count))
      (then
        (call $Term::EmptyIterator::new))
      (else
        (call $Term::TermType::TakeIterator::new (local.get $source) (local.get $count)))))

  (func $Term::TakeIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $Term::TakeIterator::get::source (local.get $self))))

  (func $Term::TakeIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::TakeIterator) (local.get $offset)))

  (func $Term::TakeIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::TakeIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::TakeIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::TakeIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::TakeIterator::new
          (local.get $substituted_source)
          (call $Term::TakeIterator::get::count (local.get $self))))))

  (func $Term::TakeIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::TakeIterator::traits::size_hint (param $self i32) (result i32)
    (local $source_length i32)
    (select
      (global.get $NULL)
      (call $Utils::i32::min_u
        (local.tee $source_length (call $Term::traits::size_hint (call $Term::TakeIterator::get::source (local.get $self))))
        (call $Term::TakeIterator::get::count (local.get $self)))
      (i32.eq (local.get $source_length) (global.get $NULL))))

  (func $Term::TakeIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $index i32)
    (local $source_state i32)
    (local $dependencies i32)
    ;; Initialize the iterator state
    (local.set $iterator_state
      (if (result i32)
        (i32.eq (local.get $iterator_state) (global.get $NULL))
        (then
          ;; If this is the first iteration, allocate a new cell to hold the iteration state
          (call $Term::TakeIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    (if (result i32 i32 i32)
      ;; If we have taken the specified number of items, dispose of the temporary iteration state cell and return the complete marker
      (i32.eq
        (local.tee $index (call $Term::TakeIterator::get::iterator_state_index (local.get $iterator_state)))
        (call $Term::TakeIterator::get::count (local.get $self)))
      (then
        (call $Term::drop (local.get $iterator_state))
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise invoke the source iterator
        (call $Term::traits::next
          (call $Term::TakeIterator::get::source (local.get $self))
          (call $Term::TakeIterator::get::iterator_state_source_state (local.get $iterator_state))
          (local.get $state))
        (local.set $dependencies)
        (local.set $source_state)
        ;; Update the iterator state with the next index and the source iterator state
        (call $Term::TakeIterator::set_iterator_state
          (local.get $iterator_state)
          (i32.add (local.get $index) (i32.const 1))
          (local.get $source_state))
        ;; The iterator value is already on the top of the stack, so emit the updated iterator state and dependencies
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $Term::TakeIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.const 2)))
    (call $Term::TakeIterator::set_iterator_state (local.get $iterator_state) (i32.const 0) (global.get $NULL)))

  (func $Term::TakeIterator::get::iterator_state_index (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $Term::TakeIterator::get::iterator_state_source_state (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $Term::TakeIterator::set_iterator_state (param $iterator_state i32) (param $index i32) (param $source_state i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $index))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $source_state))))
