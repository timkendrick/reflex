;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $HashmapValuesIterator
    (@struct $HashmapValuesIterator
      (@field $source (@ref $Term)))

    (@derive $size (@get $HashmapValuesIterator))
    (@derive $equals (@get $HashmapValuesIterator))
    (@derive $hash (@get $HashmapValuesIterator))

    (@export $HashmapValuesIterator (@get $HashmapValuesIterator)))

  (export "isHashmapValuesIterator" (func $Term::HashmapValuesIterator::is))
  (export "getHashmapValuesIteratorSource" (func $Term::HashmapValuesIterator::get::source))

  (func $Term::HashmapValuesIterator::startup)

  (func $Term::HashmapValuesIterator::new (export "createHashmapValuesIterator") (param $source i32) (result i32)
    (call $Term::TermType::HashmapValuesIterator::new (local.get $source)))

  (func $Term::HashmapValuesIterator::traits::is_atomic (param $self i32) (result i32)
    (local $source i32)
    (local $bucket_index i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Term::Hashmap::get::num_entries (local.tee $source (call $Term::HashmapValuesIterator::get::source (local.get $self)))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Term::Hashmap::get_capacity (local.get $source)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key to determine whether the current bucket is empty
            (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))
            (then
              (if
                ;; If the current bucket is not empty, and its value is non-atomic, return false
                (i32.eqz (call $Term::traits::is_atomic (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index))))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $Term::HashmapValuesIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::HashmapValuesIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        ;; TODO: Avoid unnecessary hashmap keys substitution
        (call $Term::HashmapValuesIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::HashmapValuesIterator::new (local.get $substituted_source)))))

  (func $Term::HashmapValuesIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::HashmapValuesIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::Hashmap::traits::length (call $Term::HashmapValuesIterator::get::source (local.get $self))))

  (func $Term::HashmapValuesIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $source i32)
    (local $index i32)
    (local $key i32)
    (if (result i32 i32 i32)
      ;; If we have iterated through all the buckets, return the complete marker
      (i32.eq
        (local.tee $index
          ;; Get the current iterator index from the state (initializing to zero if this is the first iteration)
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (call $Term::Hashmap::get_capacity (local.tee $source (call $Term::HashmapValuesIterator::get::source (local.get $self)))))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise inspect the next bucket
        (if (result i32 i32 i32)
          (i32.eqz (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $source) (local.get $index))))
          (then
            ;; If this is an empty bucket, skip to the next bucket
            (call $Term::HashmapValuesIterator::traits::next (local.get $self) (i32.add (local.get $index) (i32.const 1)) (local.get $state)))
          (else
            ;; Otherwise emit the bucket value and the incremented iterator state
            (call $Term::Hashmap::get_bucket_value (local.get $source) (local.get $index))
            (i32.add (local.get $index) (i32.const 1))
            (global.get $NULL)))))))
