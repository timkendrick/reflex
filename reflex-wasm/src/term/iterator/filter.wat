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
  (export "createFilterIterator" (func $Term::TermType::FilterIterator::new))
  (export "getFilterIteratorSource" (func $Term::FilterIterator::get::source))
  (export "getFilterIteratorPredicate" (func $Term::FilterIterator::get::predicate))

  (func $Term::FilterIterator::startup)

  (func $Term::FilterIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::FilterIterator::get::source (local.get $self)))))

  (func $Term::FilterIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::FilterIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::FilterIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::FilterIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $Term::FilterIterator::get::source (local.get $self))))

  (func $Term::FilterIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $Term::FilterIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
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
        ;; Otherwise apply the predicate function to the given value and evaluate the result
        (call $Term::traits::evaluate
          ;; TODO: Avoid unnecessary heap allocations for intermediate values
          (call $Term::Application::new
            (call $Term::FilterIterator::get::predicate (local.get $self))
            (call $Term::List::of (local.get $value)))
            (local.get $state))
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
