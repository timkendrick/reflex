;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; See https://craftinginterpreters.com/hash-tables.html
  (@let $Hashmap
    (@struct $Hashmap
      (@field $num_entries i32)
      (@field $buckets
        (@repeated
          (@struct $HashmapBucket
            (@field $key (@ref $Term))
            (@field $value (@ref $Term))))))

    (@derive $size (@get $Hashmap))

    (@export $Hashmap (@get $Hashmap)))

  (@apply
    (@import $HashmapMethods "./hashmap_methods.wat")
    $Term::Hashmap
    $HashmapBucket
    i32
    i32
    (call $Term::get_hash)
    (call $Term::traits::equals)
    (global.get $Term::Hashmap::MIN_UNSIZED_HASHMAP_CAPACITY)
    (call $Term::Hashmap::allocate)
    (call $Term::init))

  (export "isHashmap" (func $Term::Hashmap::is))
  (export "getHashmapNumEntries" (func $Term::Hashmap::get::num_entries))

  (export "getHashmapBucketKey" (func $Term::Hashmap::get_bucket_key))
  (export "getHashmapBucketValue" (func $Term::Hashmap::get_bucket_value))
  (export "insertHashmapEntry" (func $Term::Hashmap::insert))

  (@const $Term::Hashmap::EMPTY i32 (call $Term::TermType::Hashmap::new (i32.const 0)))

  ;; Minimum hashmap capacity when allocating non-zero-length hashmaps of unknown size
  (global $Term::Hashmap::MIN_UNSIZED_HASHMAP_CAPACITY i32 (i32.const 8))

  (func $Hashmap::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; This assumes that hashmaps with the same size and hash are almost certainly identical
    (i32.eq
      (call $Hashmap::get::num_entries (local.get $self))
      (call $Hashmap::get::num_entries (local.get $other))))

  (func $Hashmap::traits::hash (param $self i32) (param $state i64) (result i64)
    ;; FIXME: Two instances of an equivalent hashmap might return different hashes if keys were added in a different order
    ;; Consider e.g. insertion sort when initializing the buckets to ensure consistent hash
    (local $num_entries i32)
    (local $capacity i32)
    (local $index i32)
    (local $key i32)
    (local.get $state)
    ;; Hash the number of hashmap entries
    (local.tee $num_entries (call $Hashmap::get::num_entries (local.get $self)))
    (local.set $state (call $Hash::write_i32))
    ;; Hash the hashmap items
    (if (result i64)
      (i32.eqz (local.get $num_entries))
      ;; If the hashmap is empty, nothing more to do
      (then
        (local.get $state))
      (else
        ;; Hash each of the hashmap buckets
        (local.set $capacity (call $Hashmap::get::buckets::capacity (local.get $self)))
        (loop $LOOP (result i64)
          (if
            (i32.eqz
              (local.tee $key
                (call $HashmapBucket::get::key
                  (call $Hashmap::get::buckets::pointer (local.get $self) (local.get $index)))))
            ;; If this is an empty bucket then skip the hashing
            (then)
            (else
              ;; Otherwise hash the key and value
              (call $Hash::write_i64 (local.get $state) (call $Term::get_hash (local.get $key)))
              (call $HashmapBucket::get::value
                (call $Hashmap::get::buckets::pointer (local.get $self) (local.get $index)))
              (call $Term::get_hash)
              (call $Hash::write_i64)
              (local.set $state)))
          ;; If this was the final bucket return the hash, otherwise continue with the next bucket
          (if (result i64)
            (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $capacity))
            (then
              (local.get $state))
            (else
              (br $LOOP)))))))

  (func $Term::Hashmap::empty (export "createEmptyHashmap") (result i32)
    (global.get $Term::Hashmap::EMPTY))

  (func $Term::Hashmap::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the buckets pointer for an imaginary hashmap term located
    ;; at memory address 0. The pointer offset tells us how many bytes are taken up by the preceding hashmap wrapper.
    (call $Term::Hashmap::get::buckets::pointer (i32.const 0) (i32.const 0)))

  (func $Term::Hashmap::allocate (export "allocateHashmap") (param $capacity i32) (result i32)
    ;; Allocates a new Hashmap term with the given capacity, allowing key/value entries to be inserted into the allocated slots.
    ;; The hashmap must be instantiated before it can be used.
    ;; All values are stored within the hashmap array itself (open addressing) - if the ideal bucket is not empty then
    ;; the entry will be put in the next free bucket. This means that as the hashmap fills up the free slots become
    ;; increasingly scarce, so if we only allocate exactly the right number of buckets then later entries can easily
    ;; end up being stored a long way from their ideal location, resulting in poor performance for worst-case lookups.
    ;; To avoid this allocate additional hashmap capacity to prevent saturating heavily-populated bucket regions.
    (local $self i32)
    (if (result i32)
      (i32.eqz (local.get $capacity))
      (then
        ;; Return the pre-allocated singleton instance
        (call $Term::Hashmap::empty))
      (else
        ;; The standard constructor wrappers take care of allocating space for a standard term,
        ;; however they do not allocate space for extra elements as needed by the hashmap term.
        ;; This means we have to manually allocate a larger amount of space than usual,
        ;; then fill in the hashmap term contents into the newly-allocated space.
        ;; First allocate a new term wrapper with the required capacity
        (local.tee $self
          (call $Allocator::allocate
            (i32.add
              (call $Term::Hashmap::empty::sizeof)
              (i32.mul (call $HashmapBucket::sizeof) (local.get $capacity)))))
        ;; Then manually write the hashmap struct contents into the term wrapper
        (call $TermType::Hashmap::construct (call $Term::pointer::value (local.get $self)) (i32.const 0))
        (call $Term::Hashmap::set::buckets::capacity (local.get $self) (local.get $capacity))
        (call $Term::Hashmap::set::buckets::length (local.get $self) (local.get $capacity)))))

  (func $Term::Hashmap::drop (param $self i32)
    ;; Avoid dropping the global empty hashmap instance
    (if (i32.ne (local.get $self) (call $Term::Hashmap::empty))
      (then
        (call $Term::drop (local.get $self)))))

  (func $Term::Hashmap::default_capacity (export "defaultHashmapCapacity") (param $num_entries i32) (result i32)
    ;; A typical 'load factor' is 0.75 (i.e. capacity = num_entries * 4/3)
    (i32.div_u (i32.mul (local.get $num_entries) (i32.const 4)) (i32.const 3)))

  (func $Term::Hashmap::init (export "initHashmap") (param $self i32) (result i32)
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $Term::Hashmap::traits::is_atomic (param $self i32) (result i32)
    (local $bucket_index i32)
    (local $key i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Term::Hashmap::get::num_entries (local.get $self)))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Term::Hashmap::get::buckets::capacity (local.get $self)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key and use it to determine whether the current bucket is empty
            (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
            (then
              (if
                ;; If the current bucket is not empty, and its key or value is non-atomic, return false
                (i32.or
                  (i32.eqz (call $Term::traits::is_atomic (local.get $key)))
                  (i32.eqz (call $Term::traits::is_atomic (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the final bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $Term::Hashmap::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Hashmap::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Map(")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string (call $Term::Hashmap::get::num_entries (local.get $self)) (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Hashmap::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Hashmap::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Hashmap::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $num_entries i32)
    (local $capacity i32)
    (local $index i32)
    (local $key i32)
    (local $value i32)
    (local $substituted_key i32)
    (local $substituted_value i32)
    (local $results i32)
    (if (result i32)
      ;; If the hashmap is empty, return the unmodified marker
      (i32.eqz (local.tee $num_entries (call $Term::Hashmap::get::num_entries (local.get $self))))
      (then
        (global.get $NULL))
      (else
        ;; Otherwise iterate through each hashmap bucket in turn
        (local.set $capacity (call $Term::Hashmap::get::buckets::capacity (local.get $self)))
        (local.set $results (global.get $NULL))
        (loop $LOOP
          ;; Set up control flow blocks to allow breaking out of the loop and continuing with the next iteration
          (block $BREAK
            (block $CONTINUE
              (block $default
                ;; We need to cover three potential cases:
                ;;   0. We have iterated over all buckets; need to break out of the loop
                ;;   1. The current bucket is empty; need to continue with the next bucket
                ;;   2. The current bucket is not empty; need to substitute within the key and value
                ;; Get a branch condition value 0-2 that determines which of these cases to select
                (if (result i32)
                  ;; Determine whether we've reached the end of the bucket list
                  (i32.ge_u (local.get $index) (local.get $capacity))
                  (then
                    ;; We have reached the end of the bucket list, so take branch 0 (a.k.a. `break`)
                    (i32.const 0))
                  (else
                    ;; Retrieve the bucket key at the given index, and select one of two cases:
                    ;; - If that bucket key is zero (i.e. uninitialized memory), that bucket is empty so we take branch 1 (a.k.a. `continue`),
                    ;; - Otherwise the key is not empty and we take branch 2 (carry on with the body of the block)
                    (select
                      (i32.const 1)
                      (i32.const 2)
                      (i32.eqz (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $index)))))))
                ;; Branch to the correct block depending on the branch condition
                (br_table
                  ;; If we have iterated over all of the buckets, branch to the end of the loop to break out of the loop entirely
                  $BREAK
                  ;; If this is an empty bucket, skip the inner substitution by branching out of the main block
                  $CONTINUE
                  ;; Otherwise branch out of this inner block to continue with the main block
                  $default))
              ;; Get the substituted key and value
              (local.set $substituted_key
                (call $Term::traits::substitute
                  (local.get $key)
                  (local.get $variables)
                  (local.get $scope_offset)))
              (local.set $substituted_value
                (call $Term::traits::substitute
                  (local.tee $value (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $index)))
                  (local.get $variables)
                  (local.get $scope_offset)))
              (if
                ;; If the bucket was modified, and this is the first bucket to have been modified, create a new results hashmap
                (i32.and
                  (i32.or
                    (i32.ne (global.get $NULL) (local.get $substituted_key))
                    (i32.ne (global.get $NULL) (local.get $substituted_value)))
                  (i32.eq (global.get $NULL) (local.get $results)))
                (then
                  ;; Create a new result hashmap term with the correct capacity
                  (local.set $results (call $Term::Hashmap::allocate (local.get $capacity)))
                  ;; Copy any previous buckets into the results hashmap
                  (if
                    (i32.eqz (local.get $index))
                    (then)
                    (else
                      (memory.copy
                        (call $Term::Hashmap::get::buckets::pointer (local.get $results) (i32.const 0))
                        (call $Term::Hashmap::get::buckets::pointer (local.get $self) (i32.const 0))
                        (i32.mul (call $HashmapBucket::sizeof) (local.get $index)))))
                  ;; Insert the substituted entry into the results hashmap
                  (call $Term::Hashmap::insert_entry
                    (local.get $results)
                    ;; Add the unmodified key or the substituted key as appropriate
                    (select
                      (local.get $key)
                      (local.get $substituted_key)
                      (i32.eq (global.get $NULL) (local.get $substituted_key)))
                    ;; Add the unmodified value or the substituted value as appropriate
                    (select
                      (local.get $value)
                      (local.get $substituted_value)
                      (i32.eq (global.get $NULL) (local.get $substituted_value))))
                  ;; Discard the resulting number of items added to the hashmap
                  (drop))
                (else
                  ;; Otherwise if there have been modifications to the preceding buckets,
                  ;; Insert the current result into the results hashmap
                  (if
                    (i32.ne (global.get $NULL) (local.get $results))
                    (then
                      (call $Term::Hashmap::insert_entry
                        (local.get $results)
                        ;; Add the unmodified key or the substituted key as appropriate
                        (select
                          (local.get $key)
                          (local.get $substituted_key)
                          (i32.eq (global.get $NULL) (local.get $substituted_key)))
                        ;; Add the unmodified value or the substituted value as appropriate
                        (select
                          (local.get $value)
                          (local.get $substituted_value)
                          (i32.eq (global.get $NULL) (local.get $substituted_value))))
                      ;; Discard the resulting number of items added to the hashmap
                      (drop))
                    ;; Otherwise nothing more needs to be done for this bucket
                    (else)))))
            ;; Continue with the next bucket
            (local.set $index (i32.add (local.get $index) (i32.const 1)))
            (br $LOOP)))
        ;; If there were any substitutions, return the initialized results hashmap term
        (if (result i32)
          (i32.ne (global.get $NULL) (local.get $results))
          (then
            ;; Set the hashmap size
            (call $Term::Hashmap::set::num_entries (local.get $results) (local.get $num_entries))
            ;; Initialize the hashmap term
            (call $Term::init (local.get $results)))
          (else
            ;; Otherwise return the unmodified marker
            (global.get $NULL))))))

  (func $Term::Hashmap::traits::length (param $self i32) (result i32)
    (call $Term::Hashmap::get::num_entries (local.get $self)))

  (func $Term::Hashmap::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::Hashmap::traits::size_hint (param $self i32) (result i32)
    (call $Term::Hashmap::traits::length (local.get $self)))

  (func $Term::Hashmap::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
        (call $Term::Hashmap::get::buckets::capacity (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise inspect the next bucket
        (if (result i32 i32 i32)
          (i32.eqz (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $index))))
          (then
            ;; If this is an empty bucket, skip to the next bucket
            (call $Term::Hashmap::traits::next (local.get $self) (i32.add (local.get $index) (i32.const 1)) (local.get $state)))
          (else
            ;; Otherwise emit a key/value entry and the incremented iterator state
            (call $Term::List::create_pair
              (local.get $key)
              (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $index)))
            (i32.add (local.get $index) (i32.const 1))
            (global.get $NULL))))))

  (func $Term::Hashmap::traits::get (export "getHashmapValue") (param $self i32) (param $key i32) (result i32)
    (call $Term::Hashmap::retrieve (local.get $self) (local.get $key)))

  (func $Term::Hashmap::traits::has (export "hasHashmapKey") (param $self i32) (param $key i32) (result i32)
    (call $Term::Hashmap::contains_key (local.get $self) (local.get $key)))

  (func $Term::Hashmap::traits::set (export "setHashmapValue") (param $self i32) (param $key i32) (param $value i32) (result i32)
    (local $existing_bucket_index i32)
    (local $existing_capacity i32)
    (local $existing_key i32)
    (local $num_entries i32)
    (local $instance i32)
    (local $bucket_index i32)
    (if (result i32)
      ;; If the key does not already exist, return a new hashmap with an additional entry
      (i32.eq (global.get $NULL) (local.tee $existing_bucket_index (call $Term::Hashmap::find_bucket_index (local.get $self) (local.get $key))))
      (then
        ;; Allocate a new hashmap instance
        (local.tee $instance
          (call $Term::Hashmap::allocate
            (call $Term::Hashmap::default_capacity
              (i32.add
                (local.tee $num_entries (call $Term::Hashmap::get::num_entries (local.get $self)))
                (i32.const 1)))))
        ;; Copy all the existing entries across to the new hashmap
        (if
          ;; If the existing hashmap was empty, nothing to do
          (i32.eqz (local.tee $existing_capacity (call $Term::Hashmap::get::buckets::capacity (local.get $self))))
          (then)
          (else
            ;; Otherwise iterate through all the buckets of the existing hashmap
            (loop $LOOP
              ;; If the current bucket is not empty, insert the existing key and value into the new hashmap
              (if
                (local.tee $existing_key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
                (then
                  (call $Term::Hashmap::insert_entry
                    (local.get $instance)
                    (local.get $existing_key)
                    (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))
                  ;; Discard the resulting number of items added to the hashmap
                  (drop))
                (else))
              ;; If this was not the final bucket, continue with the next bucket
              (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $existing_capacity))))))
        ;; Insert the provided key and value into the new hashmap
        ;; (this function returns the number of new entries added to the hashmap)
        (call $Term::Hashmap::insert_entry (local.get $instance) (local.get $key) (local.get $value))
        ;; Keep track of how many unique entries have been added to the hashmap
        (local.set $num_entries (i32.add (local.get $num_entries)))
        ;; Set the hashmap size
        (call $Term::Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
        ;; Instantiate the hashmap term
        (call $Term::init))
      (else
        ;; Otherwise if the key already exists, return an updated hashmap with the corresponding value overridden
        (if (result i32)
          ;; If the existing value is already equal to the provided value, return the current instance
          (call $Term::traits::equals
            (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $existing_bucket_index))
            (local.get $value))
          (then
            (local.get $self))
          (else
            ;; Otherwise create a clone of the current hashmap
            (local.tee $instance (call $Term::traits::clone (local.get $self)))
            ;; Update the bucket value on the cloned hashmap
            (call $Term::Hashmap::update_bucket_value (local.get $instance) (local.get $existing_bucket_index) (local.get $value))
            ;; Instantiate the cloned hashmap
            (call $Term::init))))))

  (func $Term::Hashmap::traits::keys (param $self i32) (result i32)
    (call $Term::HashmapKeysIterator::new (local.get $self)))

  (func $Term::Hashmap::traits::values (param $self i32) (result i32)
    (call $Term::HashmapValuesIterator::new (local.get $self)))

  (func $Term::Hashmap::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a hashmap, return the existing instance
      (call $Term::Hashmap::is (local.get $iterator))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the hashmap items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (local.tee $length (call $Term::traits::size_hint (local.get $iterator))) (global.get $NULL))
          (then
            (call $Term::Hashmap::collect_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Term::Hashmap::collect_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Term::Hashmap::collect_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty hashmap
      (i32.eqz (local.get $length))
      (then
        (call $Term::Hashmap::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new hashmap to hold the results and fill it by consuming each iterator item in turn
        (local.tee $instance (call $Term::Hashmap::allocate (call $Term::Hashmap::default_capacity (local.get $length))))
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (loop $LOOP
          ;; Consume the next iterator item
          (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
          ;; Update the accumulated dependencies and iterator state
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          (if
            ;; If the iterator has been fully consumed, nothing more to do
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              ;; Otherwise resolve the current entry
              (call $Term::traits::evaluate (local.get $item) (local.get $state))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              ;; Skip over any entries which are not valid key/value pairs
              (br_if $LOOP
                (if (result i32)
                  (call $Term::List::is (local.tee $item))
                  (then
                    (i32.lt_u (call $Term::List::get_length (local.get $item)) (i32.const 2)))
                  (else
                    (i32.const 1))))
              ;; Store the item in the results hashmap
              ;; (this function returns the number of new entries added to the hashmap)
              (call $Term::Hashmap::insert_entry
                (local.get $instance)
                (call $Term::List::get_item (local.get $item) (i32.const 0))
                (call $Term::List::get_item (local.get $item) (i32.const 1)))
              ;; Keep track of how many unique entries have been added to the hashmap
              (local.set $num_entries (i32.add (local.get $num_entries)))
              ;; Continue with the next item
              (br $LOOP))))
        ;; Set the hashmap size
        (call $Term::Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
        ;; Initialize the hashmap term
        (call $Term::init)
        (local.get $dependencies))))

  (func $Term::Hashmap::collect_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; We cannot know in advance the correct size of hashmap to allocate, so we start off with the empty hashmap, then
    ;; allocate a series of hashmaps of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    ;; Start off with the empty hashmap to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $Term::Hashmap::empty))
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    (loop $LOOP
      ;; Consume the next iterator item
      (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
      ;; Update the accumulated dependencies and iterator state
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set $iterator_state)
      (if
        ;; If the iterator has been fully consumed, nothing more to do
        (i32.eq (local.tee $item) (global.get $NULL))
        (then)
        (else
          ;; Resolve the current entry
          (call $Term::traits::evaluate (local.get $item) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          ;; Skip over any entries which are not valid key/value pairs
          (br_if $LOOP
            (if (result i32)
              (call $Term::List::is (local.tee $item))
              (then
                (i32.lt_u (call $Term::List::get_length (local.get $item)) (i32.const 2)))
              (else
                (i32.const 1))))
          ;; Ensure enough capacity exists in the hashmap to store an additional entry
          (local.set $instance
            (call $Term::Hashmap::ensure_capacity
              (local.get $instance)
              (call $Term::Hashmap::default_capacity (i32.add (local.get $num_entries) (i32.const 1)))))
          ;; Store the item in the results hashmap
          ;; (this function returns the number of new entries added to the hashmap)
          (call $Term::Hashmap::insert_entry
            (local.get $instance)
            (call $Term::List::get_item (local.get $item) (i32.const 0))
            (call $Term::List::get_item (local.get $item) (i32.const 1)))
          ;; Keep track of how many unique entries have been added to the hashmap
          (local.set $num_entries (i32.add (local.get $num_entries)))
          ;; Update the hashmap length to ensure that any reallocations copy all the items collected so far
          (call $Term::Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
          ;; Continue with the next entry
          (br $LOOP))))
    (if (result i32 i32)
      ;; If the source iterator did not produce any valid entries, return the empty results hashmap as-is
      (i32.eqz (local.get $num_entries))
      (then
        (local.get $instance)
        (local.get $dependencies))
      (else
        ;; Otherwise initialize the hashmap term
        (call $Term::init (local.get $instance))
        (local.get $dependencies))))

  (func $Term::Hashmap::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a hashmap composed solely of static items, return the existing instance
      (if (result i32)
        (call $Term::Hashmap::is (local.get $iterator))
        (then
          (i32.eqz (call $Term::Hashmap::has_dynamic_entries (local.get $iterator))))
        (else
          (global.get $FALSE)))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the hashmap items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (local.tee $length (call $Term::traits::size_hint (local.get $iterator))) (global.get $NULL))
          (then
            (call $Term::Hashmap::collect_strict_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Term::Hashmap::collect_strict_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Term::Hashmap::collect_strict_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    (local $key i32)
    (local $value i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty hashmap
      (i32.eqz (local.get $length))
      (then
        (call $Term::Hashmap::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new hashmap to hold the results and fill it by consuming each iterator item in turn
        (local.set $instance (call $Term::Hashmap::allocate (call $Term::Hashmap::default_capacity (local.get $length))))
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $signal (global.get $NULL))
        (loop $LOOP
          ;; Consume the next iterator item
          (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
          ;; Update the accumulated dependencies and iterator state
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          (if
            ;; If the iterator has been fully consumed, nothing more to do
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              ;; Otherwise resolve the current entry
              (call $Term::traits::evaluate (local.get $item) (local.get $state))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $item)
              ;; Short-circuit any signals, including type error signals for any entries which are not valid key/value pairs
              (block $List
                (block $Signal
                  (block $default
                    (br_table
                      $List
                      $Signal
                      $default
                      (select
                        (i32.const 0)
                        (select
                          (i32.const 1)
                          (i32.const 2)
                          (call $Term::Signal::is (local.get $item)))
                        (call $Term::List::is (local.get $item)))))
                  (; default ;)
                  ;; Create a type error signal and fall through to the signal implementation
                  (local.set $item (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item)))))
                (; Signal ;)
                ;; Update the combined signal and continue with the next item
                (local.set $signal (call $Term::Signal::traits::union (local.get $signal) (local.get $item)))
                (br $LOOP))
              ;; Resolve the key and value
              (if (result i32 i32)
                (i32.ge_u (call $Term::List::get_length (local.get $item)) (i32.const 2))
                (then
                  (call $Term::traits::evaluate
                    (call $Term::List::get_item (local.get $item) (i32.const 0))
                    (local.get $state)))
                (else
                  (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item))
                  (global.get $NULL))))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $key)
              (if (result i32 i32)
                (i32.ge_u (call $Term::List::get_length (local.get $item)) (i32.const 2))
                (then
                  (call $Term::traits::evaluate
                    (call $Term::List::get_item (local.get $item) (i32.const 1))
                    (local.get $state)))
                (else
                  (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item))
                  (global.get $NULL))))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $value)
              ;; If the key or value resolve to a signal, or a signal has already been encountered,
              ;; update the combined signal and continue with the next item
              (br_if $LOOP
                (i32.ne
                  (global.get $NULL)
                  (local.tee $signal
                    (call $Term::Signal::traits::union
                      (local.get $signal)
                      (call $Term::Signal::traits::union
                        (select
                          (local.get $value)
                          (global.get $NULL)
                          (call $Term::Signal::is (local.get $value)))
                        (select
                          (local.get $key)
                          (global.get $NULL)
                          (call $Term::Signal::is (local.get $key))))))))
              ;; Otherwise store the item in the results hashmap
              ;; (this function returns the number of new entries added to the hashmap)
              (call $Term::Hashmap::insert_entry (local.get $instance) (local.get $key) (local.get $value))
              ;; Keep track of how many unique entries have been added to the hashmap
              (local.set $num_entries (i32.add (local.get $num_entries)))
              ;; Continue with the next item
              (br $LOOP))))
        (if (result i32 i32)
          ;; If a signal was encountered during the iteration, return the combined signal
          (i32.ne (global.get $NULL) (local.get $signal))
          (then
            (local.get $signal)
            (local.get $dependencies))
          (else
            ;; Otherwise if the source iterator did not produce any valid entries, return the empty results hashmap as-is
            (if (result i32 i32)
              (i32.eqz (local.get $num_entries))
              (then
                (local.get $instance)
                (local.get $dependencies))
              (else
                ;; Set the hashmap size
                (call $Term::Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
                ;; Initialize the results hashmap
                (call $Term::init (local.get $instance))
                (local.get $dependencies))))))))

  (func $Term::Hashmap::collect_strict_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; Given that we don't know in advance the correct size of hashmap to allocate, so we start off with the empty hashmap,
    ;; then allocate a series of hashmaps of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    (local $key i32)
    (local $value i32)
    ;; Start off with the empty hashmap to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $Term::Hashmap::empty))
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    (local.set $signal (global.get $NULL))
    (loop $LOOP
      ;; Consume the next iterator item
      (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
        ;; Update the accumulated dependencies and iterator state
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set $iterator_state)
      (if
        ;; If the iterator has been fully consumed, nothing more to do
        (i32.eq (local.tee $item) (global.get $NULL))
        (then)
        (else
          ;; Otherwise resolve the current entry
          (call $Term::traits::evaluate (local.get $item) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $item)
          ;; Short-circuit any signals, including type error signals for any entries which are not valid key/value pairs
          (block $List
            (block $Signal
              (block $default
                (br_table
                  $List
                  $Signal
                  $default
                  (select
                    (i32.const 0)
                    (select
                      (i32.const 1)
                      (i32.const 2)
                      (call $Term::Signal::is (local.get $item)))
                    (call $Term::List::is (local.get $item)))))
              (; default ;)
              ;; Create a type error signal and fall through to the signal implementation
              (local.set $item (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item)))))
            (; Signal ;)
            ;; Update the combined signal and continue with the next item
            (local.set $signal (call $Term::Signal::traits::union (local.get $signal) (local.get $item)))
            (br $LOOP))
          ;; Resolve the key and value
          (if (result i32 i32)
            (i32.ge_u (call $Term::List::get_length (local.get $item)) (i32.const 2))
            (then
              (call $Term::traits::evaluate
                (call $Term::List::get_item (local.get $item) (i32.const 0))
                (local.get $state)))
            (else
              (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item))
              (global.get $NULL))))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $key)
          (if (result i32 i32)
            (i32.ge_u (call $Term::List::get_length (local.get $item)) (i32.const 2))
            (then
              (call $Term::traits::evaluate
                (call $Term::List::get_item (local.get $item) (i32.const 1))
                (local.get $state)))
            (else
              (call $Term::Signal::of (call $Term::Condition::type_error (global.get $TermType::List) (local.get $item))
              (global.get $NULL))))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $value)
          ;; If the key or value resolve to a signal, or a signal has already been encountered,
          ;; update the combined signal and continue with the next item
          (br_if $LOOP
            (i32.ne
              (global.get $NULL)
              (local.tee $signal
                (call $Term::Signal::traits::union
                  (local.get $signal)
                  (call $Term::Signal::traits::union
                    (select
                      (local.get $value)
                      (global.get $NULL)
                      (call $Term::Signal::is (local.get $value)))
                    (select
                      (local.get $key)
                      (global.get $NULL)
                      (call $Term::Signal::is (local.get $key))))))))
          ;; Otherwise store the entry in the results hashmap, reallocating if necessary
          ;; Ensure enough capacity exists in the hashmap to store an additional entry
          (local.set $instance
            (call $Term::Hashmap::ensure_capacity
              (local.get $instance)
              (call $Term::Hashmap::default_capacity (i32.add (local.get $num_entries) (i32.const 1)))))
          ;; Store the item in the results hashmap
          ;; (this function returns the number of new entries added to the hashmap)
          (call $Term::Hashmap::insert_entry (local.get $instance) (local.get $key) (local.get $value))
          ;; Keep track of how many unique entries have been added to the hashmap
          (local.set $num_entries (i32.add (local.get $num_entries)))
          ;; Update the hashmap length to ensure that any reallocations copy all the items collected so far
          (call $Term::Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
          ;; Continue with the next item
          (br $LOOP))))
    (if (result i32 i32)
      ;; If a signal was encountered during the iteration, return the combined signal
      (i32.ne (global.get $NULL) (local.get $signal))
      (then
        (local.get $signal)
        (local.get $dependencies))
      (else
        ;; Otherwise if the source iterator did not produce any valid entries, return the empty results hashmap as-is
        (if (result i32 i32)
          (i32.eqz (local.get $num_entries))
          (then
            (local.get $instance)
            (local.get $dependencies))
          (else
            ;; Otherwise initialize the results hashmap
            (call $Term::init (local.get $instance))
            (local.get $dependencies))))))

  (func $Term::Hashmap::get_capacity (export "getHashmapCapacity") (param $self i32) (result i32)
    (call $Term::Hashmap::get::buckets::capacity (local.get $self)))

  (func $Term::Hashmap::has_dynamic_entries (param $self i32) (result i32)
    (local $bucket_index i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return false
      (i32.eqz (call $Term::Hashmap::get::num_entries (local.get $self)))
      (then
        (global.get $FALSE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Term::Hashmap::get::buckets::capacity (local.get $self)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key to determine whether the current bucket is empty
            (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))
            (then
              (if
                ;; If the current bucket is not empty, and its key or value is dynamic, return true
                (i32.or
                  (i32.eqz (call $Term::is_static (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))))
                  (i32.eqz (call $Term::is_static (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))
                (then
                  (return (global.get $TRUE)))
                (else)))
            (else))
          ;; If this was not the final bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a dynamic entry, return false
        (global.get $FALSE)))))
