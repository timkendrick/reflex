;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $FilterIterator::startup)

  (func $FilterIterator::new (export "createFilterIterator") (param $source i32) (param $predicate i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::FilterIterator) (i32.const 2)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $predicate))
    (call $Term::init))

  (func $FilterIterator::is (export "isFilterIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::FilterIterator) (call $Term::get_type (local.get $self))))

  (func $FilterIterator::get::source (export "getFilterIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $FilterIterator::get::predicate (export "getFilterIteratorPredicate") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $FilterIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $FilterIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $FilterIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $FilterIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $FilterIterator::get::source (local.get $self))
    (call $Hash::write_term)
    (call $FilterIterator::get::predicate (local.get $self))
    (call $Hash::write_term))

  (func $FilterIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $FilterIterator::get::source (local.get $self))
        (call $FilterIterator::get::source (local.get $other)))
      (call $Term::traits::equals
        (call $FilterIterator::get::predicate (local.get $self))
        (call $FilterIterator::get::predicate (local.get $other)))))

  (func $FilterIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $FilterIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $FilterIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $FilterIterator::get::source (local.get $self))))

  (func $FilterIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $FilterIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
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
          (call $Application::new
            (call $FilterIterator::get::predicate (local.get $self))
            (call $List::of (local.get $value)))
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
            (call $FilterIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies))))))))
