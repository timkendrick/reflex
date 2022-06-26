;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $SkipIterator::startup)

  (func $SkipIterator::new (export "createSkipIterator") (param $source i32) (param $count i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::SkipIterator) (i32.const 2)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $count))
    (call $Term::init))

  (func $SkipIterator::is (export "isSkipIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::SkipIterator) (call $Term::get_type (local.get $self))))

  (func $SkipIterator::get::source (export "getSkipIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $SkipIterator::get::count (export "getSkipIteratorCount") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $SkipIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $SkipIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $SkipIterator::get::source (local.get $self))))

  (func $SkipIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $SkipIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $SkipIterator::get::source (local.get $self))
    (call $Hash::write_term)
    (call $SkipIterator::get::count (local.get $self))
    (call $Hash::write_i32))

  (func $SkipIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $SkipIterator::get::source (local.get $self))
        (call $SkipIterator::get::source (local.get $other)))
      (i32.eq
        (call $SkipIterator::get::count (local.get $self))
        (call $SkipIterator::get::count (local.get $other)))))

  (func $SkipIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $SkipIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $SkipIterator::traits::size_hint (param $self i32) (result i32)
    (local $source_length i32)
    (select
      (global.get $NULL)
      (i32.sub
        (local.tee $source_length (call $Term::traits::size_hint (call $SkipIterator::get::source (local.get $self))))
        (call $SkipIterator::get::count (local.get $self)))
      (i32.eq (local.get $source_length) (global.get $NULL))))

  (func $SkipIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; Determine whether this is the first iteration
      (i32.eq (global.get $NULL) (local.get $iterator_state))
      (then
        ;; Handle the first iteration separately
        (call $SkipIterator::first (local.get $self) (local.get $state)))
      (else
        ;; Otherwise defer to the underlying source iterator implementation
        (call $Term::traits::next
          (call $SkipIterator::get::source (local.get $self))
          (local.get $iterator_state)
          (local.get $state)))))

  (func $SkipIterator::first (param $self i32) (param $state i32) (result i32 i32 i32)
    (local $iterator_state i32)
    (local $source i32)
    (local $remaining i32)
    (local $dependencies i32)
    (local.set $source (call $SkipIterator::get::source (local.get $self)))
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    (if
      (local.tee $remaining (call $SkipIterator::get::count (local.get $self)))
      (then
        ;; Consume the source iterator for the specified number of iterations, accumulating the iterator state and dependencies
        (loop $LOOP
          (call $Term::traits::next (local.get $source) (local.get $iterator_state) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          (if
            ;; If the source iterator has been fully consumed before the skip count, return the complete marker
            (i32.eq (global.get $NULL))
            (then
              (global.get $NULL)
              (global.get $NULL)
              (local.get $dependencies)
              (return))
            (else
              (br_if $LOOP (local.tee $remaining (i32.sub (local.get $remaining) (i32.const 1))))))))
      (else))
    ;; All items have been skipped; we can continue emitting source iterator values as normal
    (call $Term::traits::next (local.get $source) (local.get $iterator_state) (local.get $state))
    (call $Dependencies::traits::union (local.get $dependencies))))
