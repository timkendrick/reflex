;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $HashmapKeysIterator
    (@struct $HashmapKeysIterator
      (@field $source (@ref $Term)))

    (@derive $size (@get $HashmapKeysIterator))
    (@derive $equals (@get $HashmapKeysIterator))
    (@derive $hash (@get $HashmapKeysIterator))

    (@export $HashmapKeysIterator (@get $HashmapKeysIterator)))

  (export "isHashmapKeysIterator" (func $Term::HashmapKeysIterator::is))
  (export "getHashmapKeysIteratorSource" (func $Term::HashmapKeysIterator::get::source))

  (func $Term::HashmapKeysIterator::startup)

  (func $Term::HashmapKeysIterator::new (export "createHashmapKeysIterator") (param $source i32) (result i32)
    (call $Term::TermType::HashmapKeysIterator::new (local.get $source)))

  (func $Term::HashmapKeysIterator::traits::is_atomic (param $self i32) (result i32)
    (local $source i32)
    (local $bucket_index i32)
    (local $key i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Term::Hashmap::get::num_entries (local.tee $source (call $Term::HashmapKeysIterator::get::source (local.get $self)))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Term::Hashmap::get_capacity (local.get $source)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key and use it to determine whether the current bucket is empty
            (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
            (then
              (if
                ;; If the current bucket is not empty, and its key is non-atomic, return false
                (i32.eqz (call $Term::traits::is_atomic (local.get $key)))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $Term::HashmapKeysIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::HashmapKeysIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        ;; TODO: Avoid unnecessary hashmap values substitution
        (call $Term::HashmapKeysIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::HashmapKeysIterator::new (local.get $substituted_source)))))

  (func $Term::HashmapKeysIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::HashmapKeysIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::HashmapKeysIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::Hashmap::traits::length (call $Term::HashmapKeysIterator::get::source (local.get $self))))

  (func $Term::HashmapKeysIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
        (call $Term::Hashmap::get_capacity (local.tee $source (call $Term::HashmapKeysIterator::get::source (local.get $self)))))
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
            (call $Term::HashmapKeysIterator::traits::next (local.get $self) (i32.add (local.get $index) (i32.const 1)) (local.get $state)))
          (else
            ;; Otherwise emit the key and the incremented iterator state
            (local.get $key)
            (i32.add (local.get $index) (i32.const 1))
            (global.get $NULL)))))))
