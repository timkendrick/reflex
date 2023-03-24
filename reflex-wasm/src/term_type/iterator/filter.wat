;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $FilterIterator
    (@struct $FilterIterator
      (@field $source (@ref $Term))
      (@field $predicate (@ref $Term)))

    (@derive $size (@get $FilterIterator))
    (@derive $equals (@get $FilterIterator))
    (@derive $hash (@get $FilterIterator))

    (@export $FilterIterator (@get $FilterIterator)))

  (export "isFilterIterator" (func $Term::FilterIterator::is))
  (export "getFilterIteratorSource" (func $Term::FilterIterator::get::source))
  (export "getFilterIteratorPredicate" (func $Term::FilterIterator::get::predicate))

  (func $Term::FilterIterator::new (export "createFilterIterator") (param $source i32) (param $predicate i32) (result i32)
    (call $Term::TermType::FilterIterator::new (local.get $source) (local.get $predicate)))

  (func $Term::FilterIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::FilterIterator::get::source (local.get $self)))))

  (func $Term::FilterIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::FilterIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::FilterIterator) (local.get $offset)))

  (func $Term::FilterIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::FilterIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::FilterIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local $substituted_predicate i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::FilterIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_predicate
      (call $Term::traits::substitute
        (call $Term::FilterIterator::get::predicate (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_source))
        (i32.eq (global.get $NULL) (local.get $substituted_predicate)))
      (then
        (global.get $NULL))
      (else
        (call $Term::FilterIterator::new
          (select
            (call $Term::FilterIterator::get::source (local.get $self))
            (local.get $substituted_source)
            (i32.eq (global.get $NULL) (local.get $substituted_source)))
          (select
            (call $Term::FilterIterator::get::predicate (local.get $self))
            (local.get $substituted_predicate)
            (i32.eq (global.get $NULL) (local.get $substituted_predicate)))))))

  (func $Term::FilterIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::FilterIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $Term::FilterIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $Term::FilterIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
    (local.set $dependencies)
    (local.set $iterator_state)
    (if (result i32 i32 i32)
      ;; If the source iterator has been fully consumed, emit the complete marker
      (i32.eq (local.tee $value) (global.get $NULL))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        ;; Otherwise apply the predicate function to the source iterator item
        (call $Term::traits::apply
          (call $Term::FilterIterator::get::predicate (local.get $self))
          (call $Term::List::of (local.get $value))
          (local.get $state))
        ;; Combine the function application dependencies with the iteration dependencies
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; Evaluate the result and combine the accumulated dependencies
        (call $Term::traits::evaluate (local.get $state))
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; If the predicate returned a truthy result, emit the iterator value
        (if (result i32 i32 i32)
          (call $Term::traits::is_truthy)
          (then
            (local.get $value)
            (local.get $iterator_state)
            (local.get $dependencies))
          (else
            ;; Otherwise try the next item
            (call $Term::FilterIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies))))))))
