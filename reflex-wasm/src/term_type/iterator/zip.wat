;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $ZipIterator
    (@struct $ZipIterator
      (@field $left (@ref $Term))
      (@field $right (@ref $Term)))

    (@derive $size (@get $ZipIterator))
    (@derive $equals (@get $ZipIterator))
    (@derive $hash (@get $ZipIterator))

    (@export $ZipIterator (@get $ZipIterator)))

  (export "isZipIterator" (func $Term::ZipIterator::is))
  (export "getZipIteratorLeft" (func $Term::ZipIterator::get::left))
  (export "getZipIteratorRight" (func $Term::ZipIterator::get::right))

  (func $Term::ZipIterator::new (export "createZipIterator") (param $left i32) (param $right i32) (result i32)
    (call $Term::TermType::ZipIterator::new (local.get $left) (local.get $right)))

  (func $Term::ZipIterator::traits::is_atomic (param $self i32) (result i32)
    (if (result i32)
      (call $Term::traits::is_atomic (call $Term::ZipIterator::get::left (local.get $self)))
      (then
        (call $Term::traits::is_atomic (call $Term::ZipIterator::get::right (local.get $self))))
      (else
        (global.get $FALSE))))

  (func $Term::ZipIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::ZipIterator) (local.get $offset)))

  (func $Term::ZipIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::ZipIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::ZipIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_left i32)
    (local $substituted_right i32)
    (local.set $substituted_left
      (call $Term::traits::substitute
        (call $Term::ZipIterator::get::left (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_right
      (call $Term::traits::substitute
        (call $Term::ZipIterator::get::right (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_left))
        (i32.eq (global.get $NULL) (local.get $substituted_right)))
      (then
        (global.get $NULL))
      (else
        (call $Term::ZipIterator::new
          (select
            (call $Term::ZipIterator::get::left (local.get $self))
            (local.get $substituted_left)
            (i32.eq (global.get $NULL) (local.get $substituted_left)))
          (select
            (call $Term::ZipIterator::get::right (local.get $self))
            (local.get $substituted_right)
            (i32.eq (global.get $NULL) (local.get $substituted_right)))))))

  (func $Term::ZipIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::ZipIterator::traits::size_hint (param $self i32) (result i32)
    (local $left_length i32)
    (local $right_length i32)
    (select
      (global.get $NULL)
      (call $Utils::i32::min_u
        (local.tee $left_length (call $Term::traits::size_hint (call $Term::ZipIterator::get::left (local.get $self))))
        (local.tee $right_length (call $Term::traits::size_hint (call $Term::ZipIterator::get::right (local.get $self)))))
      (i32.and
        (i32.eq (local.get $left_length) (global.get $NULL))
        (i32.eq (local.get $right_length) (global.get $NULL)))))

  (func $Term::ZipIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
          (call $Term::ZipIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    ;; Get the next iterator item and state from the left iterator
    (call $Term::traits::next
      (call $Term::ZipIterator::get::left (local.get $self))
      (call $Term::ZipIterator::get_iterator_state_left_state (local.get $iterator_state))
      (local.get $state))
    (local.set $dependencies)
    (local.set $left_state)
    (local.set $left_value)
    ;; Get the next iterator item and state from the right iterator
    (call $Term::traits::next
      (call $Term::ZipIterator::get::right (local.get $self))
      (call $Term::ZipIterator::get_iterator_state_right_state (local.get $iterator_state))
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
        (call $Term::ZipIterator::set_iterator_state (local.get $iterator_state) (local.get $left_state) (local.get $right_state))
        (call $Term::List::create_pair (local.get $left_value) (local.get $right_value))
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $Term::ZipIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.const 2)))
    (call $Term::ZipIterator::set_iterator_state (local.get $iterator_state) (global.get $NULL) (global.get $NULL)))

  (func $Term::ZipIterator::get_iterator_state_left_state (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $Term::ZipIterator::get_iterator_state_right_state (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $Term::ZipIterator::set_iterator_state (param $iterator_state i32) (param $left_state i32) (param $right_state i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $left_state))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $right_state))))
