;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $IndexedAccessorIterator
    (@struct $IndexedAccessorIterator
      (@field $source (@ref $Term))
      (@field $index i32))

    (@derive $size (@get $IndexedAccessorIterator))
    (@derive $equals (@get $IndexedAccessorIterator))
    (@derive $hash (@get $IndexedAccessorIterator))

    (@export $IndexedAccessorIterator (@get $IndexedAccessorIterator)))

  (export "isIndexedAccessorIterator" (func $Term::IndexedAccessorIterator::is))
  (export "getIndexedAccessorIteratorSource" (func $Term::IndexedAccessorIterator::get::source))
  (export "getIndexedAccessorIteratorIndex" (func $Term::IndexedAccessorIterator::get::index))

  (func $Term::IndexedAccessorIterator::new (export "createIndexedAccessorIterator") (param $source i32) (param $index i32) (result i32)
    (call $Term::TermType::IndexedAccessorIterator::new (local.get $source) (local.get $index)))

  (func $Term::IndexedAccessorIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::IndexedAccessorIterator::get::source (local.get $self)))))

  (func $Term::IndexedAccessorIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::IndexedAccessorIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::IndexedAccessorIterator) (local.get $offset)))

  (func $Term::IndexedAccessorIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::IndexedAccessorIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::IndexedAccessorIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::IndexedAccessorIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::IndexedAccessorIterator::new
          (local.get $substituted_source)
          (call $Term::IndexedAccessorIterator::get::index (local.get $self))))))

  (func $Term::IndexedAccessorIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::IndexedAccessorIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $Term::IndexedAccessorIterator::get::source (local.get $self))))

  (func $Term::IndexedAccessorIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $Term::IndexedAccessorIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
    (local.set $dependencies)
    (local.set $iterator_state)
    (local.set $value)
    (if (result i32 i32 i32)
      ;; If the source iterator has been fully consumed, emit the complete marker
      (i32.eq (global.get $NULL) (local.get $value))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        ;; Otherwise retrieve the indexed field from the current item
        (call $Term::IndexedAccessorIterator::get_value
          (local.get $value)
          (call $Term::IndexedAccessorIterator::get::index (local.get $self))
          (local.get $state))
        ;; Update the accumulated dependencies
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; If an invalid field index was specified, return the nil term
        ;; TODO: Determine correct behaviour for invalid indexed accessor iterator indices
        (if (result i32 i32 i32)
          (i32.eq (local.tee $value) (global.get $NULL))
          (then
            (call $Term::Nil::new)
            (local.get $iterator_state)
            (local.get $dependencies))
          (else
            ;; Otherwise emit the retrieved value and the source iterator state
            (local.get $value)
            (local.get $iterator_state)
            (local.get $dependencies))))))

  (func $Term::IndexedAccessorIterator::get_value (param $target i32) (param $index i32) (param $state i32) (result i32 i32)
    (local $term_type i32)
    (local.set $term_type (call $Term::get_type (local.get $target)))
    (@switch
      (@list
        ;; If the target is a list term, retrieve the field value directly
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::List))
          (return
            (call $Term::List::get_item (local.get $target) (local.get $index))
            (global.get $NULL)))
        ;; Otherwise if the target is an iterator, retrieve the indexed item
        (@list
          (call $TermType::implements::iterate (local.get $term_type))
          (return
            (call $Term::IndexedAccessorIterator::get_iterator_value (local.get $target) (local.get $index) (local.get $state)))))
      ;; For all other term types, return the null sentinel marker
      (global.get $NULL)
      (global.get $NULL)))

  (func $Term::IndexedAccessorIterator::get_iterator_value (param $target i32) (param $index i32) (param $state i32) (result i32 i32)
    (local $value i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    ;; Initialize the iterator state
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    ;; Iterate over the source iterator to reach the desired index
    (loop $LOOP
      (call $Term::traits::next (local.get $target) (local.get $iterator_state) (local.get $state))
      ;; Update the accumulated dependencies
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      ;; Update the iterator state
      (local.set $iterator_state)
      ;; Set the stored value to the value returned by the iterator
      (local.set $value)
      ;; Determine whether more iterations are needed, leaving the result on the stack
      (i32.and
        ;; We only continue iterating if this was not the final item...
        (i32.gt_u (local.get $index) (i32.const 0))
        ;; ...and if all the iterator items have not yet been consumed
        (i32.ne (local.get $value) (global.get $NULL)))
      ;; Decrement the remaining number of items to iterate
      (local.set $index (i32.sub (local.get $index) (i32.const 1)))
      ;; Continue with the next item if this was not the final item
      (br_if $LOOP))
    ;; Return the retrieved value along with any dependencies accumulated during the iteration
    (local.get $value)
    (local.get $dependencies)))
