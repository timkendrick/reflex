;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $IntersperseIterator
    (@struct $IntersperseIterator
      (@field $source (@ref $Term))
      (@field $separator (@ref $Term)))

    (@derive $size (@get $IntersperseIterator))
    (@derive $equals (@get $IntersperseIterator))
    (@derive $hash (@get $IntersperseIterator))

    (@export $IntersperseIterator (@get $IntersperseIterator)))

  (export "isIntersperseIterator" (func $Term::IntersperseIterator::is))
  (export "getIntersperseIteratorSource" (func $Term::IntersperseIterator::get::source))
  (export "getIntersperseIteratorSeparator" (func $Term::IntersperseIterator::get::separator))

  (func $Term::IntersperseIterator::new (export "createIntersperseIterator") (param $source i32) (param $separator i32) (result i32)
    (call $Term::TermType::IntersperseIterator::new (local.get $source) (local.get $separator)))

  (func $Term::IntersperseIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::IntersperseIterator::get::source (local.get $self)))))

  (func $Term::IntersperseIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::IntersperseIterator) (local.get $offset)))

  (func $Term::IntersperseIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::IntersperseIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::IntersperseIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local $substituted_separator i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::IntersperseIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_separator
      (call $Term::traits::substitute
        (call $Term::IntersperseIterator::get::separator (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_source))
        (i32.eq (global.get $NULL) (local.get $substituted_separator)))
      (then
        (global.get $NULL))
      (else
        (call $Term::IntersperseIterator::new
          (select
            (call $Term::IntersperseIterator::get::source (local.get $self))
            (local.get $substituted_source)
            (i32.eq (global.get $NULL) (local.get $substituted_source)))
          (select
            (call $Term::IntersperseIterator::get::separator (local.get $self))
            (local.get $substituted_separator)
            (i32.eq (global.get $NULL) (local.get $substituted_separator)))))))

  (func $Term::IntersperseIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::IntersperseIterator::traits::size_hint (param $self i32) (result i32)
    (local $num_source_items i32)
    (if (result i32)
      (i32.eq
        (local.tee $num_source_items
          (call $Term::traits::size_hint
            (call $Term::IntersperseIterator::get::source (local.get $self))))
        (global.get $NULL))
      (then
        (global.get $NULL))
      (else
        (select
          (i32.const 0)
          (i32.sub
            (i32.mul (local.get $num_source_items) (i32.const 2))
            (i32.const 1))
          (i32.eqz (local.get $num_source_items))))))

  (func $Term::IntersperseIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $item i32)
    (local $dependencies i32)
    (local $inner_iterator_state i32)
    ;; If this is the first iteration, initialize the iterator state
    (if (result i32 i32 i32)
      (i32.eq (local.get $iterator_state) (global.get $NULL))
      (then
        ;; Consume the first item from the source iterator
        (call $Term::traits::next
          (call $Term::IntersperseIterator::get::source (local.get $self))
          (global.get $NULL)
          (local.get $state))
        (local.set $dependencies)
        (local.set $iterator_state)
        (if (result i32 i32 i32)
          ;; If the source iterator is empty, return the complete marker
          (i32.eq (local.tee $item) (global.get $NULL))
          (then
            (global.get $NULL)
            (global.get $NULL)
            (local.get $dependencies))
          (else
            ;; Otherwise emit the first item and and a newly-allocated iterator state
            (local.get $item)
            (call $Term::IntersperseIterator::allocate_iterator_state
              (global.get $NULL)
              (local.get $iterator_state))
            (local.get $dependencies))))
      (else
        (if (result i32 i32 i32)
          ;; If the cached next item is empty, that indicates that the next emitted item is due to be a separator
          (i32.eq
            (global.get $NULL)
            (local.tee $item
              (call $Term::IntersperseIterator::get_iterator_state_item (local.get $iterator_state))))
          (then
            ;; If all source items have been emitted, we don't want to emit another separator.
            ;; This means before emitting the separator, we consume the next source item as a lookahead,
            ;; and either bail out if the source iterator is fully consumed or store the emitted source value
            ;; as the next emission for the outer value.
            ;; Consume the next item (lookahead)
            (call $Term::traits::next
              (call $Term::IntersperseIterator::get::source (local.get $self))
              (call $Term::IntersperseIterator::get_iterator_state_state (local.get $iterator_state))
              (local.get $state))
            (local.set $dependencies)
            ;; Update the inner iterator state
            (local.set $item)
            (call $Term::IntersperseIterator::set_iterator_state_state
              (local.get $iterator_state)
              (local.get $item))
            (if (result i32 i32 i32)
              ;; If all source iterator items have been consumed, clean up the iterator state and return the complete marker
              (i32.eq (local.tee $item) (global.get $NULL))
              (then
                (call $Term::drop (local.get $iterator_state))
                (global.get $NULL)
                (global.get $NULL)
                (local.get $dependencies))
              (else
                ;; Otherwise store the lookahead item for the next iteration and emit a separator term
                (call $Term::IntersperseIterator::set_iterator_state_item (local.get $iterator_state) (local.get $item))
                (call $Term::IntersperseIterator::get::separator (local.get $self))
                (local.get $iterator_state)
                (local.get $dependencies))))
          (else
            ;; Ensure the next emission is a separator
            (call $Term::IntersperseIterator::set_iterator_state_item (local.get $iterator_state) (global.get $NULL))
            ;; Emit the cached result (stored by the previous iteration's lookahead)
            (local.get $item)
            (local.get $iterator_state)
            (global.get $NULL))))))

  (func $Term::IntersperseIterator::allocate_iterator_state (param $item i32) (param $state i32) (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.const 2)))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $item))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $state)))

  (func $Term::IntersperseIterator::get_iterator_state_item (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $Term::IntersperseIterator::set_iterator_state_item (param $iterator_state i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $value)))

  (func $Term::IntersperseIterator::get_iterator_state_state (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $Term::IntersperseIterator::set_iterator_state_state (param $iterator_state i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $value))))
