;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $FlattenIterator::startup)

  (func $FlattenIterator::new (export "createFlattenIterator") (param $source i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::FlattenIterator) (i32.const 1)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::init))

  (func $FlattenIterator::is (export "isFlattenIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::FlattenIterator) (call $Term::get_type (local.get $self))))

  (func $FlattenIterator::get::source (export "getFlattenIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $FlattenIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $FlattenIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $FlattenIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $FlattenIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $FlattenIterator::get::source (local.get $self))
    (call $Hash::write_term))

  (func $FlattenIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $FlattenIterator::get::source (local.get $self))
      (call $FlattenIterator::get::source (local.get $other))))

  (func $FlattenIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $FlattenIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $FlattenIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $FlattenIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $outer_state i32)
    (local $inner_source i32)
    (local $inner_state i32)
    (local $value i32)
    (local $dependencies i32)
    ;; If this is the first iteration, initialize the iterator state
    (if (result i32 i32)
      (i32.eq (local.get $iterator_state) (global.get $NULL))
      (then
        ;; Get the initial source iterator
        (call $Term::traits::next
          (call $FlattenIterator::get::source (local.get $self))
          (global.get $NULL)
          (local.get $state))
        (local.set $dependencies)
        (local.set $outer_state)
        (if (result i32 i32)
          ;; If the iterator of source iterators was empty, return the complete marker
          (i32.eq (local.tee $inner_source) (global.get $NULL))
          (then
            (return
              (global.get $NULL)
              (global.get $NULL)
              (local.get $dependencies)))
          (else
            ;; Otherwise allocate a new cell to hold the iteration state
            (call $FlattenIterator::allocate_iterator_state (local.get $outer_state) (local.get $inner_source))
            (local.get $dependencies))))
      (else
        ;; Otherwise use the state that was passed in from the previous iteration
        (local.get $iterator_state)
        (global.get $NULL)))
    (local.set $dependencies)
    (local.set $iterator_state)
    ;; Get the next iterator item and updated state
    (call $Term::traits::next
      (call $FlattenIterator::get_iterator_state_inner_source (local.get $iterator_state))
      (call $FlattenIterator::get_iterator_state_inner_state (local.get $iterator_state))
      (local.get $state))
    (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
    (local.set $inner_state)
    (if (result i32 i32 i32)
      ;; Check whether the inner iterator has been fully consumed
      (i32.eq (local.tee $value) (global.get $NULL))
      (then
        ;; Get the next source iterator
        (call $Term::traits::next
          (call $FlattenIterator::get::source (local.get $self))
          (call $FlattenIterator::get_iterator_state_outer_state (local.get $iterator_state))
          (local.get $state))
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        (local.set $outer_state)
        (if (result i32 i32 i32)
          ;; If this was the final source iterator, dispose of the temporary iteration state cell and return the complete marker
          (i32.eq (local.tee $inner_source) (global.get $NULL))
          (then
            (call $Term::drop (local.get $iterator_state))
            (global.get $NULL)
            (global.get $NULL)
            (local.get $dependencies))
          (else
            ;; Otherwise switch to the next source iterator and try again
            (call $FlattenIterator::set_iterator_state (local.get $iterator_state) (local.get $outer_state) (local.get $inner_source))
            (call $FlattenIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))))
      (else
        ;; Update the iterator state
        (call $FlattenIterator::set_iterator_state_inner_state (local.get $iterator_state) (local.get $inner_state))
        ;; Emit the value and the updated state
        (local.get $value)
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $FlattenIterator::allocate_iterator_state (param $outer_state i32) (param $inner_source i32) (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Cell::new (i32.const 3)))
    (call $FlattenIterator::set_iterator_state (local.get $iterator_state) (local.get $outer_state) (local.get $inner_source)))

  (func $FlattenIterator::set_iterator_state (param $iterator_state i32) (param $outer_state i32) (param $inner_source i32)
    (call $FlattenIterator::set_iterator_state_outer_state (local.get $iterator_state) (local.get $outer_state))
    (call $FlattenIterator::set_iterator_state_inner_source (local.get $iterator_state) (local.get $inner_source))
    (call $FlattenIterator::set_iterator_state_inner_state (local.get $iterator_state) (global.get $NULL)))

  (func $FlattenIterator::get_iterator_state_outer_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $FlattenIterator::set_iterator_state_outer_state (param $iterator_state i32) (param $value i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $value)))

  (func $FlattenIterator::get_iterator_state_inner_source (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $FlattenIterator::set_iterator_state_inner_source (param $iterator_state i32) (param $value i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $value)))

  (func $FlattenIterator::get_iterator_state_inner_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 2)))

  (func $FlattenIterator::set_iterator_state_inner_state (param $iterator_state i32) (param $value i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 2) (local.get $value))))
