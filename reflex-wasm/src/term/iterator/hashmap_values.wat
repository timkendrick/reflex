;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $HashmapValuesIterator::startup)

  (func $HashmapValuesIterator::new (export "createHashmapValuesIterator") (param $source i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::HashmapValuesIterator) (i32.const 1)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::init))

  (func $HashmapValuesIterator::is (export "isHashmapValuesIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::HashmapValuesIterator) (call $Term::get_type (local.get $self))))

  (func $HashmapValuesIterator::get::source (export "getHashmapValuesIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $HashmapValuesIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $HashmapValuesIterator::traits::is_atomic (param $self i32) (result i32)
    (local $source i32)
    (local $bucket_index i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Hashmap::get::num_entries (local.tee $source (call $HashmapValuesIterator::get::source (local.get $self)))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Hashmap::get_capacity (local.get $source)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key to determine whether the current bucket is empty
            (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))
            (then
              (if
                ;; If the current bucket is not empty, and its value is non-atomic, return false
                (i32.eqz (call $Term::traits::is_atomic (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index))))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $HashmapValuesIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $HashmapValuesIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $HashmapValuesIterator::get::source (local.get $self))
    (call $Hash::write_term))

  (func $HashmapValuesIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $HashmapValuesIterator::get::source (local.get $self))
      (call $HashmapValuesIterator::get::source (local.get $other))))

  (func $HashmapValuesIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $HashmapValuesIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $HashmapValuesIterator::traits::size_hint (param $self i32) (result i32)
    (call $Hashmap::traits::length (call $HashmapValuesIterator::get::source (local.get $self))))

  (func $HashmapValuesIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
        (call $Hashmap::get_capacity (local.tee $source (call $HashmapValuesIterator::get::source (local.get $self)))))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise inspect the next bucket
        (if (result i32 i32 i32)
          (i32.eqz (local.tee $key (call $Hashmap::get_bucket_key (local.get $source) (local.get $index))))
          (then
            ;; If this is an empty bucket, skip to the next bucket
            (call $HashmapValuesIterator::traits::next (local.get $self) (i32.add (local.get $index) (i32.const 1)) (local.get $state)))
          (else
            ;; Otherwise emit the bucket value and the incremented iterator state
            (call $Hashmap::get_bucket_value (local.get $source) (local.get $index))
            (i32.add (local.get $index) (i32.const 1))
            (global.get $NULL)))))))
