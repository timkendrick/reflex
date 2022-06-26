;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; See https://craftinginterpreters.com/hash-tables.html
  ;; Minimum hashmap capacity when allocating non-zero-length hashmaps of unknown size
  (global $Hashmap::MIN_UNSIZED_HASHMAP_CAPACITY i32 (i32.const 8))
  (global $Hashmap::NUM_HEADER_FIELDS i32 (i32.const 1))
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Hashmap::EMPTY (mut i32) (i32.const -1))

  (func $Hashmap::startup
    ;; Pre-allocate the singleton instance
    (local $instance i32)
    ;; Allocate a new struct of the required size and type (one field for the number of entries; no additional items)
    (local.tee $instance (call $Term::new (global.get $TermType::Hashmap) (global.get $Hashmap::NUM_HEADER_FIELDS)))
    ;; Store the number of entries as the first field
    (call $Term::set_field (local.get $instance) (i32.const 0) (i32.const 0))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $Hashmap::EMPTY))

  (func $Hashmap::allocate (export "allocateHashmap") (param $capacity i32) (result i32)
    ;; Allocates a new Hashmap term with the given capacity, allowing key/value entries to be inserted into the allocated slots.
    ;; The hashmap must be instantiated before it can be used.
    ;; All values are stored within the hashmap array itself (open addressing) - if the ideal bucket is not empty then
    ;; the entry will be put in the next free bucket. This means that as the hashmap fills up the free slots become
    ;; increasingly scarce, so if we only allocate exactly the right number of buckets then later entries can easily
    ;; end up being stored a long way from their ideal location, resulting in poor performance for worst-case lookups.
    ;; To avoid this allocate additional hashmap capacity to prevent saturating heavily-populated bucket regions.
    (local $self i32)
    (if (result i32)
      (i32.eq (local.get $capacity) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Hashmap::EMPTY))
      (else
        ;; Allocate a new struct of the required size and type (one field for the number of entries, plus two fields per key/value entry)
        (local.tee $self (call $Term::new (global.get $TermType::Hashmap) (i32.add (global.get $Hashmap::NUM_HEADER_FIELDS) (i32.mul (local.get $capacity) (i32.const 2))))))))

  (func $Hashmap::default_capacity (export "defaultHashmapCapacity") (param $num_entries i32) (result i32)
    ;; A typical 'load factor' is 0.75 (i.e. capacity = num_entries * 4/3)
    (i32.div_u (i32.mul (local.get $num_entries) (i32.const 4)) (i32.const 3)))

  (func $Hashmap::init (export "initHashmap") (param $self i32) (param $num_entries i32) (result i32)
    ;; This assumes the given hashmap has already been allocated and filled with items
    ;; Store the length as the second field
    (call $Hashmap::set::num_entries (local.get $self) (local.get $num_entries))
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $Hashmap::empty (export "createEmptyHashmap") (result i32)
    ;; Allocate a new hashmap of the required length
    ;; (this will return the pre-allocated empty hashmap singleton)
    (call $Hashmap::allocate (i32.const 0)))

  (func $Hashmap::is (export "isHashmap") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Hashmap) (call $Term::get_type (local.get $term))))

  (func $Hashmap::get::num_entries (export "getHashmapNumEntries") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Hashmap::set::num_entries (param $self i32) (param $value i32)
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value)))

  (func $Hashmap::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Hashmap::traits::is_atomic (param $self i32) (result i32)
    (local $bucket_index i32)
    (local $key i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Hashmap::get::num_entries (local.get $self)))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Hashmap::get_capacity (local.get $self)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key and use it to determine whether the current bucket is empty
            (local.tee $key (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
            (then
              (if
                ;; If the current bucket is not empty, and its key or value is non-atomic, return false
                (i32.or
                  (i32.eqz (call $Term::traits::is_atomic (local.get $key)))
                  (i32.eqz (call $Term::traits::is_atomic (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $Hashmap::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Hashmap::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; FIXME: Two instances of an equivalent hashmap might return different hashes if keys were added in a different order
    ;; Consider e.g. insertion sort when initializing the buckets to ensure consistent hash
    (local $capacity i32)
    (local $index i32)
    (local $key i32)
    (local.get $state)
    ;; Hash the number of hashmap entries
    (call $Hashmap::get::num_entries (local.get $self))
    (local.set $state (call $Hash::write_i32))
    ;; Hash the hashmap items
    (if (result i32)
      (i32.eq (i32.const 0) (local.tee $capacity (call $Hashmap::get_capacity (local.get $self))))
      ;; If the hashmap is empty, nothing more to do
      (then
        (local.get $state))
      (else
        ;; Hash each of the hashmap buckets
        (loop $LOOP (result i32)
          (if
            (i32.eqz (local.tee $key (call $Hashmap::get_bucket_key (local.get $self) (local.get $index))))
            ;; If this is an empty bucket then skip the hashing
            (then)
            (else
              ;; Otherwise hash the key and value
              (local.get $state)
              (local.get $key)
              (call $Hash::write_term)
              (call $Hashmap::get_bucket_value (local.get $self) (local.get $index))
              (local.set $state (call $Hash::write_term))))
          ;; If this was the final bucket return the hash, otherwise continue with the next bucket
          (if (result i32)
            (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $capacity))
            (then
              (local.get $state))
            (else
              (br $LOOP)))))))

  (func $Hashmap::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    ;; (this makes the assumption that hashmaps with the same size and hash are almost certainly identical)
    (i32.and
      (i32.eq (call $Hashmap::get::num_entries (local.get $self)) (call $Hashmap::get::num_entries (local.get $other)))
      (i32.eq (call $Term::get_hash (local.get $self)) (call $Term::get_hash (local.get $other)))))

  (func $Hashmap::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Hashmap::traits::length (param $self i32) (result i32)
    (call $Hashmap::get::num_entries (local.get $self)))

  (func $Hashmap::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Hashmap::traits::size_hint (param $self i32) (result i32)
    (call $Hashmap::traits::length (local.get $self)))

  (func $Hashmap::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
        (call $Hashmap::get_capacity (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise inspect the next bucket
        (if (result i32 i32 i32)
          (i32.eqz (local.tee $key (call $Hashmap::get_bucket_key (local.get $self) (local.get $index))))
          (then
            ;; If this is an empty bucket, skip to the next bucket
            (call $Hashmap::traits::next (local.get $self) (i32.add (local.get $index) (i32.const 1)) (local.get $state)))
          (else
            ;; Otherwise emit a key/value entry and the incremented iterator state
            (call $List::create_pair
              (local.get $key)
              (call $Hashmap::get_bucket_value (local.get $self) (local.get $index)))
            (i32.add (local.get $index) (i32.const 1))
            (global.get $NULL))))))

  (func $Hashmap::traits::get (export "getHashmapValue") (param $self i32) (param $key i32) (result i32)
    (local $bucket_index i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $bucket_index (call $Hashmap::find_bucket_index (local.get $self) (local.get $key))))
      (then
        (global.get $NULL))
      (else
        (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))

  (func $Hashmap::traits::has (export "hasHashmapKey") (param $self i32) (param $key i32) (result i32)
    (i32.ne (call $Hashmap::find_bucket_index (local.get $self) (local.get $key)) (global.get $NULL)))

  (func $Hashmap::traits::set (export "setHashmapValue") (param $self i32) (param $key i32) (param $value i32) (result i32)
    (local $existing_bucket_index i32)
    (local $existing_capacity i32)
    (local $existing_key i32)
    (local $num_entries i32)
    (local $instance i32)
    (local $bucket_index i32)
    (if (result i32)
      ;; If the key does not already exist, return a new hashmap with an additional entry
      (i32.eq (global.get $NULL) (local.tee $existing_bucket_index (call $Hashmap::find_bucket_index (local.get $self) (local.get $key))))
      (then
        ;; Allocate a new hashmap instance
        (local.tee $instance (call $Hashmap::allocate (call $Hashmap::default_capacity (local.tee $num_entries (i32.add (call $Hashmap::get::num_entries (local.get $self)) (i32.const 1))))))
        ;; Copy all the existing entries across to the new hashmap
        (if
          ;; If the existing hashmap was empty, nothing to do
          (i32.eqz (local.tee $existing_capacity (call $Hashmap::get_capacity (local.get $self))))
          (then)
          (else
            ;; Otherwise iterate through all the buckets of the existing hashmap
            (loop $LOOP
              ;; If the current bucket is not empty, insert the existing key and value into the new hashmap
              (if
                (local.tee $existing_key (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
                (then
                  (call $Hashmap::insert
                    (local.get $instance)
                    (local.get $existing_key)
                    (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index))))
                (else))
              ;; If this was not the last bucket, continue with the next bucket
              (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $existing_capacity))))))
        ;; Insert the provided key and value into the new hashmap
        (call $Hashmap::insert (local.get $instance) (local.get $key) (local.get $value))
        ;; Instantiate the new hashmap
        (call $Hashmap::init (local.get $num_entries)))
      (else
        ;; Otherwise if the key already exists, return an updated hashmap with the corresponding value overridden
        (if (result i32)
          ;; If the existing value is already equal to the provided value, return the current instance
          (call $Term::traits::equals
            (call $Hashmap::get_bucket_value (local.get $self) (local.get $existing_bucket_index))
            (local.get $value))
          (then
            (local.get $self))
          (else
            ;; Otherwise create a clone of the current hashmap
            (local.tee $instance (call $Term::traits::clone (local.get $self)))
            ;; Update the bucket value on the cloned hashmap
            (call $Hashmap::update_bucket_value (local.get $instance) (local.get $existing_bucket_index) (local.get $value))
            ;; Instantiate the cloned hashmap
            (call $Term::init))))))

  (func $Hashmap::traits::keys (param $self i32) (result i32)
    (call $HashmapKeysIterator::new (local.get $self)))

  (func $Hashmap::traits::values (param $self i32) (result i32)
    (call $HashmapValuesIterator::new (local.get $self)))

  (func $Hashmap::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a hashmap, return the existing instance
      (call $Hashmap::is (local.get $iterator))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the hashmap items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (local.tee $length (call $Term::traits::size_hint (local.get $iterator))) (global.get $NULL))
          (then
            (call $Hashmap::collect_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Hashmap::collect_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Hashmap::collect_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty hashmap
      (i32.eqz (local.get $length))
      (then
        (call $Hashmap::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new hashmap to hold the results and fill it by consuming each iterator item in turn
        (local.tee $instance (call $Hashmap::allocate (call $Hashmap::default_capacity (local.get $length))))
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
                  (call $List::is (local.tee $item))
                  (then
                    (i32.lt_u (call $List::get::length (local.get $item)) (i32.const 2)))
                  (else
                    (i32.const 1))))
              ;; Store the item in the results hashmap
              (call $Hashmap::insert
                (local.get $instance)
                (call $List::get_item (local.get $item) (i32.const 0))
                (call $List::get_item (local.get $item) (i32.const 1)))
              ;; Keep track of how many entries have been added to the hashmap
              (local.set $num_entries (i32.add (local.get $num_entries) (i32.const 1)))
              ;; Continue with the next item
              (br $LOOP))))
        ;; Initialize the hashmap term
        (call $Hashmap::init (local.get $num_entries))
        (local.get $dependencies))))

  (func $Hashmap::collect_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; We cannot know in advance the correct size of hashmap to allocate, so we start off with the empty hashmap, then
    ;; allocate a series of hashmaps of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    ;; Start off with the empty hashmap to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $Hashmap::empty))
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
              (call $List::is (local.tee $item))
              (then
                (i32.lt_u (call $List::get::length (local.get $item)) (i32.const 2)))
              (else
                (i32.const 1))))
          (if
            ;; If the current hashmap has reached the default load factor, reallocate a new hashmap with larger capacity
            (i32.ge_u
              (call $Hashmap::default_capacity (i32.add (local.get $num_entries) (i32.const 1)))
              (local.get $capacity))
            (then
              ;; Assign the hashmap length to ensure that the reallocation copies all the items collected so far
              (call $Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
              ;; Reallocate the hashmap to a new location with double the capacity
              (local.set $instance
                (call $Hashmap::reallocate
                  (local.get $instance)
                  (local.tee $capacity
                    (select
                      ;; If this is the first non-empty allocation, create a hashmap of a predetermined capacity
                      (global.get $Hashmap::MIN_UNSIZED_HASHMAP_CAPACITY)
                      ;; Otherwise create a new hashmap with double the existing capacity
                      ;; (this ensures amortized hashmap allocations as the number of items increases)
                      (i32.mul (local.get $capacity) (i32.const 2))
                      (i32.eqz (local.get $capacity)))))))
            (else))
          ;; Store the item in the results hashmap
          (call $Hashmap::insert
            (local.get $instance)
            (call $List::get_item (local.get $item) (i32.const 0))
            (call $List::get_item (local.get $item) (i32.const 0)))
          ;; Keep track of how many entries have been added to the hashmap
          (local.set $num_entries (i32.add (local.get $num_entries) (i32.const 1)))
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
        (call $Hashmap::init (local.get $instance) (local.get $num_entries))
        (local.get $dependencies))))

  (func $Hashmap::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a hashmap composed solely of static items, return the existing instance
      (if (result i32)
        (call $Hashmap::is (local.get $iterator))
        (then
          (i32.eqz (call $Hashmap::has_dynamic_entries (local.get $iterator))))
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
            (call $Hashmap::collect_strict_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Hashmap::collect_strict_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Hashmap::collect_strict_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
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
        (call $Hashmap::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new hashmap to hold the results and fill it by consuming each iterator item in turn
        (local.set $instance (call $Hashmap::allocate (call $Hashmap::default_capacity (local.get $length))))
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
                          (call $Signal::is (local.get $item)))
                        (call $List::is (local.get $item)))))
                  (; default ;)
                  ;; Create a type error signal and fall through to the signal implementation
                  (local.set $item (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))))
                (; Signal ;)
                ;; Update the combined signal and continue with the next item
                (local.set $signal (call $Signal::traits::union (local.get $signal) (local.get $item)))
                (br $LOOP))
              ;; Resolve the key and value
              (if (result i32 i32)
                (i32.ge_u (call $List::get::length (local.get $item)) (i32.const 2))
                (then
                  (call $Term::traits::evaluate
                    (call $List::get_item (local.get $item) (i32.const 0))
                    (local.get $state)))
                (else
                  (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))
                  (global.get $NULL)))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $key)
              (if (result i32 i32)
                (i32.ge_u (call $List::get::length (local.get $item)) (i32.const 2))
                (then
                  (call $Term::traits::evaluate
                    (call $List::get_item (local.get $item) (i32.const 1))
                    (local.get $state)))
                (else
                  (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))
                  (global.get $NULL)))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $value)
              ;; If the key or value resolve to a signal, or a signal has already been encountered,
              ;; update the combined signal and continue with the next item
              (br_if $LOOP
                (i32.ne
                  (global.get $NULL)
                  (local.tee $signal
                    (call $Signal::traits::union
                      (local.get $signal)
                      (call $Signal::traits::union
                        (select
                          (local.get $value)
                          (global.get $NULL)
                          (call $Signal::is (local.get $value)))
                        (select
                          (local.get $key)
                          (global.get $NULL)
                          (call $Signal::is (local.get $key))))))))
              ;; Otherwise store the item in the results hashmap
              (call $Hashmap::insert (local.get $instance) (local.get $key) (local.get $value))
              ;; Keep track of how many entries have been added to the hashmap
              (local.set $num_entries (i32.add (local.get $num_entries) (i32.const 1)))
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
                (call $Hashmap::init (local.get $instance) (local.get $num_entries))
                (local.get $dependencies))))))))

  (func $Hashmap::collect_strict_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; Given that we don't know in advance the correct size of hashmap to allocate, so we start off with the empty hashmap,
    ;; then allocate a series of hashmaps of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $num_entries i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    (local $key i32)
    (local $value i32)
    ;; Start off with the empty hashmap to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $Hashmap::empty))
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
                      (call $Signal::is (local.get $item)))
                    (call $List::is (local.get $item)))))
              (; default ;)
              ;; Create a type error signal and fall through to the signal implementation
              (local.set $item (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))))
            (; Signal ;)
            ;; Update the combined signal and continue with the next item
            (local.set $signal (call $Signal::traits::union (local.get $signal) (local.get $item)))
            (br $LOOP))
          ;; Resolve the key and value
          (if (result i32 i32)
            (i32.ge_u (call $List::get::length (local.get $item)) (i32.const 2))
            (then
              (call $Term::traits::evaluate
                (call $List::get_item (local.get $item) (i32.const 0))
                (local.get $state)))
            (else
              (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))
              (global.get $NULL)))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $key)
          (if (result i32 i32)
            (i32.ge_u (call $List::get::length (local.get $item)) (i32.const 2))
            (then
              (call $Term::traits::evaluate
                (call $List::get_item (local.get $item) (i32.const 1))
                (local.get $state)))
            (else
              (call $Signal::of (call $Condition::type_error (global.get $TermType::List) (local.get $item)))
              (global.get $NULL)))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $value)
          ;; If the key or value resolve to a signal, or a signal has already been encountered,
          ;; update the combined signal and continue with the next item
          (br_if $LOOP
            (i32.ne
              (global.get $NULL)
              (local.tee $signal
                (call $Signal::traits::union
                  (local.get $signal)
                  (call $Signal::traits::union
                    (select
                      (local.get $value)
                      (global.get $NULL)
                      (call $Signal::is (local.get $value)))
                    (select
                      (local.get $key)
                      (global.get $NULL)
                      (call $Signal::is (local.get $key))))))))
          ;; Otherwise store the entry in the results hashmap, reallocating if necessary
          (if
            ;; If the current hashmap has reached the default load factor, reallocate a new hashmap with larger capacity
            (i32.ge_u
              (call $Hashmap::default_capacity (i32.add (local.get $num_entries) (i32.const 1)))
              (local.get $capacity))
            (then
              ;; Assign the hashmap length to ensure that the reallocation copies all the items collected so far
              (call $Hashmap::set::num_entries (local.get $instance) (local.get $num_entries))
              ;; Reallocate the hashmap to a new location with double the capacity
              (local.set $instance
                (call $Hashmap::reallocate
                  (local.get $instance)
                  (local.tee $capacity
                    (select
                      ;; If this is the first non-empty allocation, create a hashmap of a predetermined capacity
                      (global.get $Hashmap::MIN_UNSIZED_HASHMAP_CAPACITY)
                      ;; Otherwise create a new hashmap with double the existing capacity
                      ;; (this ensures amortized hashmap allocations as the number of items increases)
                      (i32.mul (local.get $capacity) (i32.const 2))
                      (i32.eqz (local.get $capacity)))))))
            (else))
          ;; Store the entry in the results hashmap
          (call $Hashmap::insert (local.get $instance) (local.get $key) (local.get $value))
          ;; Keep track of how many entries have been added to the hashmap
          (local.set $num_entries (i32.add (local.get $num_entries) (i32.const 1)))
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
            (call $Hashmap::init (local.get $instance) (local.get $num_entries))
            (local.get $dependencies))))))

  (func $Hashmap::get_capacity (export "getHashmapCapacity") (param $self i32) (result i32)
    ;; Determine the hashmap capacity based on the allocated term size
    (i32.div_u (i32.sub (call $Term::get_num_fields (local.get $self)) (global.get $Hashmap::NUM_HEADER_FIELDS)) (i32.const 2)))

  (func $Hashmap::insert (export "insertHashmapEntry") (param $self i32) (param $key i32) (param $value i32)
    ;; Note that this does not grow the hashmap or update the length, it merely inserts an entry into an already-allocated slot.
    ;; If there is no free capacity available this will loop infinitely searching for an empty bucket.
    (call $Hashmap::update_bucket
      (local.get $self)
      (call $Hashmap::find_empty_bucket_index (local.get $self) (call $Term::get_hash (local.get $key)))
      (local.get $key)
      (local.get $value)))

  (func $Hashmap::reallocate (param $self i32) (param $capacity i32) (result i32)
    (local $instance i32)
    (local $source_capacity i32)
    (local $bucket_index i32)
    (local $key i32)
    (local $num_entries i32)
    (if (result i32)
      ;; If the hashmap already has sufficient capacity, return it as-is
      (i32.ge_u (local.tee $source_capacity (call $Hashmap::get_capacity (local.get $self))) (local.get $capacity))
      (then
        (local.get $self))
      (else
        ;; Otherwise allocate a new hashmap with the given capacity, and copy the contents of the source hashmap
        (local.tee $instance (call $Hashmap::allocate (local.get $capacity)))
        ;; If the source hashmap contains any entries, copy them across to the new hashmap
        (if
          (i32.eqz (call $Hashmap::get::num_entries (local.get $self)))
          (then)
          (else
            ;; Iterate through each bucket in turn, inserting the existing entries into the new hashmap
            (loop $LOOP
              (if
                ;; If the current bucket is not empty, copy it into the new hashmap
                (local.tee $key (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
                (then
                  (call $Hashmap::insert
                    (local.get $instance)
                    (local.get $key)
                    (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))
                  ;; Keep track of how many entries have been added to the hashmap
                  (local.set $num_entries (i32.add (local.get $num_entries) (i32.const 1))))
                (else))
              ;; If this was not the last bucket, continue with the next bucket
              (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $source_capacity))))))
        ;; Rewrite the source hashmap as a redirect pointer term
        ;; (this is to avoid breaking any existing pointers to the original hashmap address)
        (call $Term::redirect (local.get $self) (local.get $instance))
        ;; Initialize the hashmap term
        ;; TODO: investigate whether reallocated hashmap initialization can be moved to the parent function
        (call $Hashmap::init (local.get $num_entries)))))

  (func $Hashmap::has_dynamic_entries (param $self i32) (result i32)
    (local $bucket_index i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return false
      (i32.eqz (call $Hashmap::get::num_entries (local.get $self)))
      (then
        (global.get $FALSE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Hashmap::get_capacity (local.get $self)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key to determine whether the current bucket is empty
            (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))
            (then
              (if
                ;; If the current bucket is not empty, and its key or value is dynamic, return true
                (i32.or
                  (i32.eqz (call $Term::traits::is_static (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index))))
                  (i32.eqz (call $Term::traits::is_static (call $Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))
                (then
                  (return (global.get $TRUE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a dynamic entry, return false
        (global.get $FALSE))))

  (func $Hashmap::find_empty_bucket_index (param $self i32) (param $hash i32) (result i32)
    (local $capacity i32)
    (local $bucket_index i32)
    (local.set $capacity (call $Hashmap::get_capacity (local.get $self)))
    ;; Determine the starting field offset guess based on the hash of the specified key
    (local.set $bucket_index (call $Hashmap::get_hash_bucket (local.get $capacity) (local.get $hash)))
    ;; Iterate through the hashmap buckets from the initial offset until an empty bucket is located
    (loop $LOOP (result i32)
      (if (result i32)
        ;; If the current bucket is empty then we're good to go
        (i32.eqz (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
        (then
          (local.get $bucket_index))
        (else
          ;; Otherwise try the next bucket (wrapping around to the beginning)
          (local.set $bucket_index (i32.rem_u (i32.add (local.get $bucket_index) (i32.const 1)) (local.get $capacity)))
          (br $LOOP)))))

  (func $Hashmap::get_hash_bucket (param $capacity i32) (param $hash i32) (result i32)
    ;; Divide hashes evenly across the total bucket capacity via the modulo operation
    (i32.rem_u (local.get $hash) (local.get $capacity)))

  (func $Hashmap::get_bucket_key (param $self i32) (param $index i32) (result i32)
    ;; Extract the key from the given bucket
    (call $Term::get_field (local.get $self) (i32.add (call $Hashmap::get_bucket_field_offset (local.get $index)) (i32.const 0))))

  (func $Hashmap::get_bucket_value (param $self i32) (param $index i32) (result i32)
    ;; Extract the value from the given bucket
    (call $Term::get_field (local.get $self) (i32.add (call $Hashmap::get_bucket_field_offset (local.get $index)) (i32.const 1))))

  (func $Hashmap::update_bucket (param $self i32) (param $index i32) (param $key i32) (param $value i32)
    (call $Hashmap::update_bucket_key (local.get $self) (local.get $index) (local.get $key))
    (call $Hashmap::update_bucket_value (local.get $self) (local.get $index) (local.get $value)))

  (func $Hashmap::update_bucket_key (param $self i32) (param $index i32) (param $key i32)
    (call $Term::set_field
      (local.get $self)
      (i32.add (call $Hashmap::get_bucket_field_offset (local.get $index)) (i32.const 0))
      (local.get $key)))

  (func $Hashmap::update_bucket_value (param $self i32) (param $index i32) (param $value i32)
    (call $Term::set_field
      (local.get $self)
      (i32.add (call $Hashmap::get_bucket_field_offset (local.get $index)) (i32.const 1))
      (local.get $value)))

  (func $Hashmap::get_bucket_field_offset (param $index i32) (result i32)
    (i32.add (global.get $Hashmap::NUM_HEADER_FIELDS) (i32.mul (local.get $index) (i32.const 2))))

  (func $Hashmap::find_bucket_index (param $self i32) (param $key i32) (result i32)
    (local $capacity i32)
    (local $bucket_index i32)
    (local $remaining_buckets i32)
    (local $stored_key i32)
    (local.set $capacity (call $Hashmap::get_capacity (local.get $self)))
    (if (result i32)
      (i32.eqz (local.get $capacity))
      (then
        (global.get $NULL))
      (else
        (local.set $bucket_index (call $Hashmap::get_hash_bucket (local.get $capacity) (call $Term::get_hash (local.get $key))))
        (local.set $remaining_buckets (i32.add (local.get $capacity) (i32.const 1)))
        (loop $LOOP (result i32)
          (if (result i32)
            ;; If all buckets have been probed unsucessfully, return the null sentinel value
            (i32.eqz (local.tee $remaining_buckets (i32.sub (local.get $remaining_buckets) (i32.const 1))))
            (then
              (global.get $NULL))
            (else
              (local.set $stored_key (call $Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
              (if (result i32)
                ;; Check whether the key stored in the current bucket matches the provided key
                (if (result i32)
                  (i32.eqz (local.get $stored_key))
                  (then
                    ;; Ensure that empty buckets are not counted as matches
                    (global.get $FALSE))
                  (else
                    (call $Term::traits::equals(local.get $stored_key) (local.get $key))))
                (then
                  ;; Key matches; return bucket index
                  (local.get $bucket_index))
                (else
                  ;; Try the next bucket (wrapping around to the beginning)
                  (local.set $bucket_index (i32.rem_u (i32.add (local.get $bucket_index) (i32.const 1)) (local.get $capacity)))
                  (br $LOOP))))))))))
