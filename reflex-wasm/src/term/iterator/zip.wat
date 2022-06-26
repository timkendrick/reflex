;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $ZipIterator::startup)

  (func $ZipIterator::new (export "createZipIterator") (param $left i32) (param $right i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::ZipIterator) (i32.const 2)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $left))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $right))
    (call $Term::init))

  (func $ZipIterator::is (export "isZipIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::ZipIterator) (call $Term::get_type (local.get $self))))

  (func $ZipIterator::get::left (export "getZipIteratorLeft") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $ZipIterator::get::right (export "getZipIteratorRight") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $ZipIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $ZipIterator::traits::is_atomic (param $self i32) (result i32)
    (if (result i32)
      (call $Term::traits::is_atomic (call $ZipIterator::get::left (local.get $self)))
      (then
        (call $Term::traits::is_atomic (call $ZipIterator::get::right (local.get $self))))
      (else
        (global.get $FALSE))))

  (func $ZipIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $ZipIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $ZipIterator::get::left (local.get $self))
    (call $Hash::write_term)
    (call $ZipIterator::get::right (local.get $self))
    (call $Hash::write_term))

  (func $ZipIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (if (result i32)
      (call $Term::traits::equals
        (call $ZipIterator::get::left (local.get $self))
        (call $ZipIterator::get::left (local.get $other)))
      (then
        (call $Term::traits::equals
          (call $ZipIterator::get::right (local.get $self))
          (call $ZipIterator::get::right (local.get $other))))
      (else
        (global.get $FALSE))))

  (func $ZipIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $ZipIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $ZipIterator::traits::size_hint (param $self i32) (result i32)
    (local $left_length i32)
    (local $right_length i32)
    (select
      (global.get $NULL)
      (call $Utils::i32::min_u
        (local.tee $left_length (call $Term::traits::size_hint (call $ZipIterator::get::left (local.get $self))))
        (local.tee $right_length (call $Term::traits::size_hint (call $ZipIterator::get::right (local.get $self)))))
      (i32.or
        (i32.eq (local.get $left_length) (global.get $NULL))
        (i32.eq (local.get $right_length) (global.get $NULL)))))

  (func $ZipIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $left_value i32)
    (local $left_state i32)
    (local $right_value i32)
    (local $right_state i32)
    (local $dependencies i32)
    ;; Initialize the iterator state
    (local.set $iterator_state
      (if (result i32)
        (i32.eq (local.get $iterator_state) (global.get $NULL))
        (then
          ;; If this is the first iteration, allocate a new cell to hold the iteration state
          (call $ZipIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    ;; Get the next iterator item and state from the left iterator
    (call $Term::traits::next
      (call $ZipIterator::get::left (local.get $self))
      (call $ZipIterator::get_iterator_state_left_state (local.get $iterator_state))
      (local.get $state))
    (local.set $dependencies)
    (local.set $left_state)
    (local.set $left_value)
    ;; Get the next iterator item and state from the right iterator
    (call $Term::traits::next
      (call $ZipIterator::get::right (local.get $self))
      (call $ZipIterator::get_iterator_state_right_state (local.get $iterator_state))
      (local.get $state))
    (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
    (local.set $right_state)
    (local.set $right_value)
    ;; Emit the combined iterator value
    (if (result i32 i32 i32)
      (i32.or
        (i32.eq (global.get $NULL) (local.get $left_value))
        (i32.eq (global.get $NULL) (local.get $right_value)))
      (then
        ;; If one or more of the source iterators has been fully consumed,
        ;; dispose of the temporary iteration state cell and return the complete marker
        (call $Term::drop (local.get $iterator_state))
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        ;; Otherwise update the iterator state and emit the values and the updated state
        (call $ZipIterator::set_iterator_state (local.get $iterator_state) (local.get $left_state) (local.get $right_state))
        (call $List::create_pair (local.get $left_value) (local.get $right_value))
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $ZipIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Cell::new (i32.const 2)))
    (call $ZipIterator::set_iterator_state (local.get $iterator_state) (global.get $NULL) (global.get $NULL)))

  (func $ZipIterator::get_iterator_state_left_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $ZipIterator::get_iterator_state_right_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $ZipIterator::set_iterator_state (param $iterator_state i32) (param $left_state i32) (param $right_state i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $left_state))
    (call $Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $right_state))))
