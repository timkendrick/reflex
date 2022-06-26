;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $TakeIterator::startup)

  (func $TakeIterator::new (export "createTakeIterator") (param $source i32) (param $count i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::TakeIterator) (i32.const 2)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $count))
    (call $Term::init))

  (func $TakeIterator::is (export "isTakeIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::TakeIterator) (call $Term::get_type (local.get $self))))

  (func $TakeIterator::get::source (export "getTakeIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $TakeIterator::get::count (export "getTakeIteratorCount") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $TakeIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $TakeIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $TakeIterator::get::source (local.get $self))))

  (func $TakeIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $TakeIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $TakeIterator::get::source (local.get $self))
    (call $Hash::write_term)
    (call $TakeIterator::get::count (local.get $self))
    (call $Hash::write_i32))

  (func $TakeIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $TakeIterator::get::source (local.get $self))
        (call $TakeIterator::get::source (local.get $other)))
      (i32.eq
        (call $TakeIterator::get::count (local.get $self))
        (call $TakeIterator::get::count (local.get $other)))))

  (func $TakeIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $TakeIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $TakeIterator::traits::size_hint (param $self i32) (result i32)
    (local $source_length i32)
    (select
      (global.get $NULL)
      (call $Utils::i32::min_u
        (local.tee $source_length (call $Term::traits::size_hint (call $TakeIterator::get::source (local.get $self))))
        (call $TakeIterator::get::count (local.get $self)))
      (i32.eq (local.get $source_length) (global.get $NULL))))

  (func $TakeIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $index i32)
    (local $source_state i32)
    (local $dependencies i32)
    ;; Initialize the iterator state
    (local.set $iterator_state
      (if (result i32)
        (i32.eq (local.get $iterator_state) (global.get $NULL))
        (then
          ;; If this is the first iteration, allocate a new cell to hold the iteration state
          (call $TakeIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    (if (result i32 i32 i32)
      ;; If we have taken the specified number of items, dispose of the temporary iteration state cell and return the complete marker
      (i32.eq
        (local.tee $index (call $TakeIterator::get::iterator_state_index (local.get $iterator_state)))
        (call $TakeIterator::get::count (local.get $self)))
      (then
        (call $Term::drop (local.get $iterator_state))
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise invoke the source iterator
        (call $Term::traits::next
          (call $TakeIterator::get::source (local.get $self))
          (call $TakeIterator::get::iterator_state_source_state (local.get $iterator_state))
          (local.get $state))
        (local.set $dependencies)
        (local.set $source_state)
        ;; Update the iterator state with the next index and the source iterator state
        (call $TakeIterator::set_iterator_state
          (local.get $iterator_state)
          (i32.add (local.get $index) (i32.const 1))
          (local.get $source_state))
        ;; The iterator value is already on the top of the stack, so emit the updated iterator state and dependencies
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $TakeIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Cell::new (i32.const 2)))
    (call $TakeIterator::set_iterator_state (local.get $iterator_state) (i32.const 0) (global.get $NULL)))

  (func $TakeIterator::get::iterator_state_index (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $TakeIterator::get::iterator_state_source_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $TakeIterator::set_iterator_state (param $iterator_state i32) (param $index i32) (param $source_state i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $index))
    (call $Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $source_state))))
