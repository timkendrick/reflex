;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $FlattenIterator
    (@struct $FlattenIterator
      (@field $source (@ref $Term)))

    (@derive $size (@get $FlattenIterator))
    (@derive $equals (@get $FlattenIterator))
    (@derive $hash (@get $FlattenIterator))

    (@export $FlattenIterator (@get $FlattenIterator)))

  (export "isFlattenIterator" (func $Term::FlattenIterator::is))
  (export "getFlattenIteratorSource" (func $Term::FlattenIterator::get::source))

  (func $Term::FlattenIterator::new (export "createFlattenIterator") (param $source i32) (result i32)
    (call $Term::TermType::FlattenIterator::new (local.get $source)))

  (func $Term::FlattenIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::FlattenIterator::get::source (local.get $self)))))

  (func $Term::FlattenIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::FlattenIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::FlattenIterator) (local.get $offset)))

  (func $Term::FlattenIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::FlattenIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::FlattenIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::FlattenIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::FlattenIterator::new (local.get $substituted_source)))))

  (func $Term::FlattenIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::FlattenIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $Term::FlattenIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $outer_state i32)
    (local $inner_iterator i32)
    (local $inner_state i32)
    (local $value i32)
    (local $dependencies i32)
    ;; If this is the first iteration, initialize the iterator state
    (if (result i32 i32)
      (i32.eq (local.get $iterator_state) (global.get $NULL))
      (then
        ;; Get the initial item from the outer iterator
        (call $Term::traits::next
          (call $Term::FlattenIterator::get::source (local.get $self))
          (global.get $NULL)
          (local.get $state))
        (local.set $dependencies)
        (local.set $outer_state)
        (local.tee $inner_iterator)
        (if (result i32 i32)
          ;; If the outer iterator was empty, return the complete marker
          (i32.eq (global.get $NULL))
          (then
            (return
              (global.get $NULL)
              (global.get $NULL)
              (local.get $dependencies)))
          (else
            ;; Otherwise allocate a new cell to hold the overall iteration state
            (call $Term::FlattenIterator::allocate_iterator_state (local.get $outer_state) (local.get $inner_iterator))
            (local.get $dependencies))))
      (else
        ;; Otherwise use the state that was passed in from the previous iteration
        (local.get $iterator_state)
        (global.get $NULL)))
    (local.set $dependencies)
    (local.set $iterator_state)
    ;; Get the current inner iterator from the overall iterator state
    (local.tee $inner_iterator (call $Term::FlattenIterator::get_iterator_state_inner_iterator (local.get $iterator_state)))
    (if (result i32 i32 i32)
      ;; If the outer iterator has been fully consumed, dispose of the temporary iteration state cell and return the complete marker
      (i32.eq (global.get $NULL))
      (then
        (call $Term::drop (local.get $iterator_state))
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        ;; Otherwise check whether the current inner iterator is a signal result
        (if (result i32 i32 i32)
          (call $Term::Signal::is (local.get $inner_iterator))
          (then
            ;; If the current inner iterator is a signal result, emit the signal and move onto the next inner iterator
            (local.set $value (local.get $inner_iterator))
            ;; Advance the outer iterator to get the next inner iterator
            (call $Term::traits::next
              (call $Term::FlattenIterator::get::source (local.get $self))
              (call $Term::FlattenIterator::get_iterator_state_outer_state (local.get $iterator_state))
              (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (local.set $outer_state)
            (local.set $inner_iterator)
            ;; Update the overall iterator state in preparation for the next inner iterator
            (call $Term::FlattenIterator::set_iterator_state
              (local.get $iterator_state)
              (local.get $outer_state)
              (local.get $inner_iterator)
              (global.get $NULL))
            ;; Emit the signal and the updated overall iterator state
            (local.get $value)
            (local.get $iterator_state)
            (local.get $dependencies))
          (else
            ;; Otherwise if there exists a valid inner iterator, get the next inner iterator item and updated state
            (call $Term::traits::next
              (local.get $inner_iterator)
              (call $Term::FlattenIterator::get_iterator_state_inner_state (local.get $iterator_state))
              (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (local.set $inner_state)
            (local.tee $value)
            (if (result i32 i32 i32)
              ;; Check whether the inner iterator has been fully consumed
              (i32.eq (global.get $NULL))
              (then
                ;; If the inner iterator has been fully consumed, advance the outer iterator to get the next inner iterator
                (call $Term::traits::next
                  (call $Term::FlattenIterator::get::source (local.get $self))
                  (call $Term::FlattenIterator::get_iterator_state_outer_state (local.get $iterator_state))
                  (local.get $state))
                (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
                (local.set $outer_state)
                (local.set $inner_iterator)
                ;; Switch to the next inner iterator and try again
                (call $Term::FlattenIterator::set_iterator_state
                  (local.get $iterator_state)
                  (local.get $outer_state)
                  (local.get $inner_iterator)
                  (global.get $NULL))
                (call $Term::FlattenIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies)))
              (else
                ;; Update the inner iterator state
                (call $Term::FlattenIterator::set_iterator_state_inner_state
                  (local.get $iterator_state)
                  (local.get $inner_state))
                ;; Emit the value and the updated state
                (local.get $value)
                (local.get $iterator_state)
                (local.get $dependencies))))))))

  (func $Term::FlattenIterator::allocate_iterator_state (param $outer_state i32) (param $inner_iterator i32) (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.const 3)))
    (call $Term::FlattenIterator::set_iterator_state
      (local.get $iterator_state)
      (local.get $outer_state)
      (local.get $inner_iterator)
      (global.get $NULL)))

  (func $Term::FlattenIterator::set_iterator_state (param $self i32) (param $outer_state i32) (param $inner_iterator i32) (param $inner_state i32)
    (call $Term::FlattenIterator::set_iterator_state_outer_state (local.get $self) (local.get $outer_state))
    (call $Term::FlattenIterator::set_iterator_state_inner_iterator (local.get $self) (local.get $inner_iterator))
    (call $Term::FlattenIterator::set_iterator_state_inner_state (local.get $self) (local.get $inner_state)))

  (func $Term::FlattenIterator::get_iterator_state_outer_state (param $self i32) (result i32)
    (call $Term::Cell::get_field (local.get $self) (i32.const 0)))

  (func $Term::FlattenIterator::set_iterator_state_outer_state (param $self i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $self) (i32.const 0) (local.get $value)))

  (func $Term::FlattenIterator::get_iterator_state_inner_iterator (param $self i32) (result i32)
    (call $Term::Cell::get_field (local.get $self) (i32.const 1)))

  (func $Term::FlattenIterator::set_iterator_state_inner_iterator (param $self i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $self) (i32.const 1) (local.get $value)))

  (func $Term::FlattenIterator::get_iterator_state_inner_state (param $self i32) (result i32)
    (call $Term::Cell::get_field (local.get $self) (i32.const 2)))

  (func $Term::FlattenIterator::set_iterator_state_inner_state (param $self i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $self) (i32.const 2) (local.get $value))))
