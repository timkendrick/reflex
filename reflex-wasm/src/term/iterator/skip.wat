;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $SkipIterator
    (@struct $SkipIterator
      (@field $source (@ref $Term))
      (@field $count i32))

    (@derive $size (@get $SkipIterator))
    (@derive $equals (@get $SkipIterator))
    (@derive $hash (@get $SkipIterator))

    (@export $SkipIterator (@get $SkipIterator)))

  (export "isSkipIterator" (func $Term::SkipIterator::is))
  (export "getSkipIteratorSource" (func $Term::SkipIterator::get::source))
  (export "getSkipIteratorCount" (func $Term::SkipIterator::get::count))

  (func $Term::SkipIterator::startup)

  (func $Term::SkipIterator::new (export "createSkipIterator") (param $source i32) (param $count i32) (result i32)
    (if (result i32)
      (i32.eqz (local.get $count))
      (then
        (local.get $source))
      (else
        (if (result i32)
          (i32.ge_u (local.get $count) (call $Term::traits::size_hint (local.get $source)))
          (then
            (call $Term::EmptyIterator::new))
          (else
            (call $Term::TermType::SkipIterator::new (local.get $source) (local.get $count)))))))

  (func $Term::SkipIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $Term::SkipIterator::get::source (local.get $self))))

  (func $Term::SkipIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::SkipIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::SkipIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::SkipIterator::new
          (local.get $substituted_source)
          (call $Term::SkipIterator::get::count (local.get $self))))))

  (func $Term::SkipIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::SkipIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::SkipIterator::traits::size_hint (param $self i32) (result i32)
    (local $source_length i32)
    (select
      (global.get $NULL)
      (i32.sub
        (local.tee $source_length (call $Term::traits::size_hint (call $Term::SkipIterator::get::source (local.get $self))))
        (call $Term::SkipIterator::get::count (local.get $self)))
      (i32.eq (local.get $source_length) (global.get $NULL))))

  (func $Term::SkipIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; Determine whether this is the first iteration
      (i32.eq (global.get $NULL) (local.get $iterator_state))
      (then
        ;; Handle the first iteration separately
        (call $Term::SkipIterator::first (local.get $self) (local.get $state)))
      (else
        ;; Otherwise defer to the underlying source iterator implementation
        (call $Term::traits::next
          (call $Term::SkipIterator::get::source (local.get $self))
          (local.get $iterator_state)
          (local.get $state)))))

  (func $Term::SkipIterator::first (param $self i32) (param $state i32) (result i32 i32 i32)
    (local $iterator_state i32)
    (local $source i32)
    (local $remaining i32)
    (local $dependencies i32)
    (local.set $source (call $Term::SkipIterator::get::source (local.get $self)))
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    (if
      (local.tee $remaining (call $Term::SkipIterator::get::count (local.get $self)))
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
