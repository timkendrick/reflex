;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@export $HashmapMethods
    (@template
      $name $bucket_type $key_type $value_type $key_hash $key_equals $min_dynamic_capacity $allocate_with_capacity $init
      (@block
        (func (@concat "$" (@get $name) "::insert") (param $self i32) (param $key (@get $key_type)) (param $value (@get $value_type))
          ;; Note that this does not increase the allocated hashmap capacity, it merely inserts an entry into an
          ;; already-allocated bucket and updates the hashmap length if necessary.
          ;; If there is no free capacity available this will loop infinitely searching for an empty bucket.
          (if
            (call (@concat "$" (@get $name) "::insert_entry") (local.get $self) (local.get $key) (local.get $value))
            (then
              ;; If the insertion method returned a non-zero result, increment the stored number of hashmap entries
              (call (@concat "$" (@get $name) "::set::num_entries")
                (local.get $self)
                (i32.add (call (@concat "$" (@get $name) "::get::num_entries") (local.get $self)) (i32.const 1))))))

        (func (@concat "$" (@get $name) "::insert_entry") (param $self i32) (param $key (@get $key_type)) (param $value (@get $value_type)) (result i32)
          ;; Note that this does not increase the allocated hashmap capacity or update the length, it merely inserts an entry
          ;; into an already-allocated bucket.
          ;; If there is no free capacity available this will loop infinitely searching for an empty bucket.
          ;; This will return 0 if the key was already present in the hashmap, or 1 if a new key was added to the hashmap
          (local $key_already_exists i32)
          (local $bucket_index i32)
          ;; Find an empty space to insert the value, or use an existing value if one has already been allocated for this key
          (call (@concat "$" (@get $name) "::find_insertion_bucket_index_for_key") (local.get $self) (local.get $key))
          (local.set $key_already_exists)
          (local.set $bucket_index)
          (call (@concat "$" (@get $name) "::update_bucket")
            (local.get $self)
            (local.get $bucket_index)
            (local.get $key)
            (local.get $value))
          ;; Return 0 if the key already existed, or 1 if a new key was added to the hashmap
          (i32.eqz (local.get $key_already_exists)))

        (func (@concat "$" (@get $name) "::contains_key") (param $self i32) (param $key (@get $key_type)) (result i32)
          (i32.ne
            (global.get $NULL)
            (call (@concat "$" (@get $name) "::find_bucket_index") (local.get $self) (local.get $key))))

        (func (@concat "$" (@get $name) "::retrieve") (param $self i32) (param $key (@get $key_type)) (result (@get $value_type))
          (local $bucket_index i32)
          (if (result i32)
            (i32.eq (global.get $NULL) (local.tee $bucket_index (call (@concat "$" (@get $name) "::find_bucket_index") (local.get $self) (local.get $key))))
            (then
              (global.get $NULL))
            (else
              (call (@concat "$" (@get $name) "::get_bucket_value") (local.get $self) (local.get $bucket_index)))))

        (func (@concat "$" (@get $name) "::ensure_capacity") (param $self i32) (param $capacity i32) (result i32)
          (local $existing_capacity i32)
          (if (result i32)
            ;; If the current hashmap has insufficient capacity, reallocate a new hashmap with larger capacity
            (i32.lt_u (local.tee $existing_capacity (call (@concat "$" (@get $name) "::get::buckets::capacity") (local.get $self))) (local.get $capacity))
            (then
              ;; Reallocate the hashmap to a new location with double the capacity
              (call (@concat "$" (@get $name) "::clone_with_capacity")
                (local.get $self)
                (select
                  ;; If this is the first non-empty allocation, create a hashmap of a predetermined capacity
                  (@get $min_dynamic_capacity)
                  ;; Otherwise create a new hashmap with double the existing capacity
                  ;; (this ensures amortized hashmap allocations as the number of items increases)
                  (i32.mul (local.get $existing_capacity) (i32.const 2))
                  (i32.eqz (local.get $existing_capacity)))))
            (else
              (local.get $self))))

        (func (@concat "$" (@get $name) "::clone_with_capacity") (param $self i32) (param $capacity i32) (result i32)
          ;; Return a newly-allocated hashmap with the same contents and the requested capacity.
          ;; The requested capacity MUST be sufficient to hold the existing hashmap contents.
          (local $instance i32)
          (local $source_capacity i32)
          (local $bucket_index i32)
          (local $key (@get $key_type))
          (local $num_entries i32)
          ;; Allocate a new hashmap with the given capacity, and copy the contents of the source hashmap
          (local.get $capacity)
          (@get $allocate_with_capacity)
          (local.tee $instance)
          ;; If the source hashmap contains any entries, copy them across to the new hashmap
          (if
            (i32.ne
              (local.tee $num_entries (call (@concat "$" (@get $name) "::get::num_entries") (local.get $self)))
              (i32.const 0))
            (then
              ;; Determine the capacity of the source hashmap for use in the iterator termination condition
              (local.set $source_capacity (call (@concat "$" (@get $name) "::get::buckets::capacity") (local.get $self)))
              ;; Iterate through each bucket in turn, inserting the existing entries into the new hashmap
              (loop $LOOP
                ;; If the current bucket is not empty, copy it into the new hashmap
                (if
                  ;; Determine whether the bucket is empty
                  (@instruction
                    (@concat (@get $key_type) ".eqz")
                    (local.tee $key (call (@concat "$" (@get $name) "::get_bucket_key") (local.get $self) (local.get $bucket_index))))
                  ;; If the bucket is empty, nothing more to
                  (then)
                  (else
                    ;; Otherwise insert the value into the hashmap
                    ;; (this function returns the number of new entries added to the hashmap)
                    (call (@concat "$" (@get $name) "::insert_entry")
                      (local.get $instance)
                      (local.get $key)
                      (call (@concat "$" (@get $name) "::get_bucket_value") (local.get $self) (local.get $bucket_index)))
                    ;; Discard the resulting number of items added to the hashmap
                    (drop)))
                ;; If this was not the final bucket, continue with the next bucket
                (br_if $LOOP
                  (i32.lt_u
                    (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1)))
                    (local.get $source_capacity))))))
          ;; Set the hashmap size
          (call (@concat "$" (@get $name) "::set::num_entries") (local.get $instance) (local.get $num_entries))
          ;; Initialize the hashmap term
          ;; TODO: investigate whether reallocated hashmap initialization can be moved to the parent function
          (@get $init))

        (func (@concat "$" (@get $name) "::find_insertion_bucket_index_for_key") (param $self i32) (param $key (@get $key_type)) (result i32 i32)
          ;; Find the bucket index for the provided index if one has already been allocated,
          ;; otherwise return the index of a empty bucket to use for this key
          ;; The first result will be the bucket index, the second result will be 0 if the key is not already present in the
          ;; hashmap, or 1 if the key already exists
          (local $capacity i32)
          (local $bucket_index i32)
          (local $existing_key (@get $key_type))
          (local.set $capacity (call (@concat "$" (@get $name) "::get::buckets::capacity") (local.get $self)))
          ;; Determine the starting field offset guess based on the hash of the specified key
          (local.set $bucket_index (call (@concat "$" (@get $name) "::get_hash_bucket") (local.get $capacity) (local.get $key)))
          ;; Iterate through the hashmap buckets from the initial offset until an empty bucket is located
          (loop $LOOP (result i32 i32)
            (if (result i32 i32)
              ;; If the current bucket is empty, or if it has already been allocated for this particular key, then we're good to go
              (if (result i32)
                (@instruction
                  (@concat (@get $key_type) ".eqz")
                  (local.tee $existing_key (call (@concat "$" (@get $name) "::get_bucket_key") (local.get $self) (local.get $bucket_index))))
                (then
                  (i32.const 1))
                (else
                  (local.get $key)
                  (local.get $existing_key)
                  (@get $key_equals)))
              (then
                ;; We have reached an empty bucket or a bucket that has been previously allocated for the requested key
                (local.get $bucket_index)
                (call $Utils::bool::not
                  (@instruction (@concat (@get $key_type) ".eqz") (local.get $existing_key))))
              (else
                ;; Otherwise try the next bucket (wrapping around to the beginning)
                (local.set $bucket_index (i32.rem_u (i32.add (local.get $bucket_index) (i32.const 1)) (local.get $capacity)))
                (br $LOOP)))))

        (func (@concat "$" (@get $name) "::get_hash_bucket") (param $capacity i32) (param $key (@get $key_type)) (result i32)
          (local.get $key)
          (@get $key_hash)
          ;; Divide hashes evenly across the total bucket capacity via the modulo operation
          (i32.wrap_i64 (i64.rem_u (i64.extend_i32_u (local.get $capacity)))))

        (func (@concat "$" (@get $name) "::get_bucket_key") (param $self i32) (param $index i32) (result (@get $key_type))
          (call (@concat "$" (@get $bucket_type) "::get::key")
            (call (@concat "$" (@get $name) "::get::buckets::pointer") (local.get $self) (local.get $index))))

        (func (@concat "$" (@get $name) "::get_bucket_value") (param $self i32) (param $index i32) (result (@get $value_type))
          (call (@concat "$" (@get $bucket_type) "::get::value")
            (call (@concat "$" (@get $name) "::get::buckets::pointer") (local.get $self) (local.get $index))))

        (func (@concat "$" (@get $name) "::update_bucket") (param $self i32) (param $index i32) (param $key (@get $key_type)) (param $value (@get $value_type))
          (call (@concat "$" (@get $name) "::update_bucket_key") (local.get $self) (local.get $index) (local.get $key))
          (call (@concat "$" (@get $name) "::update_bucket_value") (local.get $self) (local.get $index) (local.get $value)))

        (func (@concat "$" (@get $name) "::update_bucket_key") (param $self i32) (param $index i32) (param $value (@get $key_type))
          (call (@concat "$" (@get $bucket_type) "::set::key")
            (call (@concat "$" (@get $name) "::get::buckets::pointer") (local.get $self) (local.get $index))
            (local.get $value)))

        (func (@concat "$" (@get $name) "::update_bucket_value") (param $self i32) (param $index i32) (param $value (@get $value_type))
          (call (@concat "$" (@get $bucket_type) "::set::value")
            (call (@concat "$" (@get $name) "::get::buckets::pointer") (local.get $self) (local.get $index))
            (local.get $value)))

        (func (@concat "$" (@get $name) "::find_bucket_index") (param $self i32) (param $key (@get $key_type)) (result i32)
          (local $capacity i32)
          (local $bucket_index i32)
          (local $remaining_buckets i32)
          (local $stored_key (@get $key_type))
          (local.set $capacity (call (@concat "$" (@get $name) "::get::buckets::capacity") (local.get $self)))
          (if (result i32)
            (i32.eqz (local.get $capacity))
            (then
              (global.get $NULL))
            (else
              (local.set $bucket_index (call (@concat "$" (@get $name) "::get_hash_bucket") (local.get $capacity) (local.get $key)))
              (local.set $remaining_buckets (i32.add (local.get $capacity) (i32.const 1)))
              (loop $LOOP (result i32)
                (if (result i32)
                  ;; If all buckets have been probed unsucessfully, return the null sentinel value
                  (i32.eqz (local.tee $remaining_buckets (i32.sub (local.get $remaining_buckets) (i32.const 1))))
                  (then
                    (global.get $NULL))
                  (else
                    (local.set $stored_key (call (@concat "$" (@get $name) "::get_bucket_key") (local.get $self) (local.get $bucket_index)))
                    (if (result i32)
                      (@instruction (@concat (@get $key_type) ".eqz") (local.get $stored_key))
                      (then
                        ;; If we have reached an empty bucket, there cannot be a matching bucket
                        (global.get $NULL))
                      (else
                        ;; Check whether the key stored in the current bucket matches the provided key
                        (local.get $stored_key)
                        (local.get $key)
                        (@get $key_equals)
                        (if (result i32)
                          (then
                            ;; Key matches; return bucket index
                            (local.get $bucket_index))
                          (else
                            ;; Try the next bucket (wrapping around to the beginning)
                            (local.set $bucket_index (i32.rem_u (i32.add (local.get $bucket_index) (i32.const 1)) (local.get $capacity)))
                            (br $LOOP)))))))))))))))
