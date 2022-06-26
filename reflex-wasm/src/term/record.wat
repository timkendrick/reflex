;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Record
    (@struct $Record
      (@field $keys (@ref $Term))
      (@field $values (@ref $Term))
      (@field $lookup_table (@ref $Term @optional)))

    (@derive $size (@get $Record))

    (@export $Record (@get $Record)))

  (export "isRecord" (func $Term::Record::is))
  (export "getRecordKeys" (func $Term::Record::get::keys))
  (export "getRecordValues" (func $Term::Record::get::values))

  (func $Record::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $Record::get::keys (local.get $self))
        (call $Record::get::keys (local.get $other)))
      (call $Term::traits::equals
        (call $Record::get::values (local.get $self))
        (call $Record::get::values (local.get $other)))))

  (func $Record::traits::hash (param $self i32) (param $state i32) (result i32)
    (call $Record::get::values (local.get $self))
    (call $Record::get::keys (local.get $self))
    (local.get $state)
    (call $Term::traits::hash)
    (call $Term::traits::hash))

  ;; Define the minimum number of fields at which to allocate an internal hashmap lookup table
  (global $Term::Record::LOOKUP_TABLE_MIN_SIZE i32 (i32.const 16))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Term::Record::EMPTY (mut i32) (i32.const -1))

  (func $Term::Record::startup
    ;; Pre-allocate the singleton instances
    (global.set $Term::Record::EMPTY
      (call $Term::TermType::Record::new
        (call $Term::List::empty)
        (call $Term::List::empty)
        (global.get $NULL))))

  (func $Term::Record::new (export "createRecord") (param $keys i32) (param $values i32) (result i32)
    (local $self i32)
    (if (result i32)
      (i32.eq (call $Term::List::traits::length (local.get $keys)) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::Record::EMPTY))
      (else
        (call $Term::TermType::Record::new
          (local.get $keys)
          (local.get $values)
          ;; Determine whether to allocate a lookup hashmap depending on the number of record fields
          (if (result i32)
            (i32.ge_u (call $Term::List::traits::length (local.get $keys)) (global.get $Term::Record::LOOKUP_TABLE_MIN_SIZE))
            (then
              (call $Term::Record::create_hashmap_lookup (local.get $keys) (local.get $values)))
            (else
              (global.get $NULL)))))))

  (func $Term::Record::empty (result i32)
    (global.get $Term::Record::EMPTY))

  (func $Term::Record::traits::is_atomic (param $self i32) (result i32)
    (i32.and
      (call $Term::List::traits::is_atomic (call $Term::Record::get::keys (local.get $self)))
      (call $Term::List::traits::is_atomic (call $Term::Record::get::values (local.get $self)))))

  (func $Term::Record::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Record::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_keys i32)
    (local $substituted_values i32)
    (local.set $substituted_keys
      (call $Term::traits::substitute
        (call $Term::Record::get::keys (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_values
      (call $Term::traits::substitute
        (call $Term::Record::get::values (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_keys))
        (i32.eq (global.get $NULL) (local.get $substituted_values)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Record::new
          (select
            (call $Term::Record::get::keys (local.get $self))
            (local.get $substituted_keys)
            (i32.eq (global.get $NULL) (local.get $substituted_keys)))
          (select
            (call $Term::Record::get::values (local.get $self))
            (local.get $substituted_values)
            (i32.eq (global.get $NULL) (local.get $substituted_values)))))))

  (func $Term::Record::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $keys i32)
    (local $values i32)
    (local $num_keys i32)
    (local $num_invalid_keys i32)
    (local $index i32)
    (local $item i32)
    ;; TODO: Decide correct behavior when JSON-serializing objects with non-serializable keys
    ;; Determine the number of unserializable keys in order to subtract them from the total
    (if
      (local.tee $num_keys (call $Term::List::get_length (local.tee $keys (call $Term::Record::get::keys (local.get $self)))))
      (then
        (loop $LOOP
          (local.set $num_invalid_keys
            (select
              (local.get $num_invalid_keys)
              (i32.add (local.get $num_invalid_keys) (i32.const 1))
              (call $Term::Record::is_valid_json_key (call $Term::List::get_item (local.get $keys) (local.get $index)))))
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $num_keys)))))
      (else))
    (local.set $index (i32.const 0))
    (local.set $values (call $Term::Record::get::values (local.get $self)))
    (if (result i32 i32)
      ;; If the record is empty, write an empty JSON object literal
      (i32.eqz (i32.sub (local.get $num_keys) (local.get $num_invalid_keys)))
      (then
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Allocate two bytes for opening and closing braces and write the characters to the output
        (call $Allocator::extend (local.get $offset) (i32.const 2))
        (i32.store8 offset=0 (local.get $offset) (@char "{"))
        (i32.store8 offset=1 (local.get $offset) (@char "}"))
        ;; Return the number of bytes written
        (i32.add (local.get $offset) (i32.const 2)))
      (else
        ;; Allocate one byte for the opening brace and write the character to the output
        (call $Allocator::extend (local.get $offset) (i32.const 1))
        (i32.store8 (local.get $offset) (@char "{"))
        ;; Iterate through the list items
        (loop $LOOP
          (if
            ;; If the current key is invalid, skip to the next one
            (i32.eqz (call $Term::Record::is_valid_json_key (local.tee $item (call $Term::List::get_item (local.get $keys) (local.get $index)))))
            (then
              ;; Decrement the number of remaining invalid keys
              (local.set $num_invalid_keys (i32.sub (local.get $num_invalid_keys) (i32.const 1)))
              ;; Continue with the next item (there must be more valid keys remaining in order to get here)
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))
            (else
              ;; Write the current key to the output and store the updated offset
              (local.set $offset
                (call $Term::traits::to_json
                  (local.get $item)
                  ;; The target offset is incremented to reflect the preceding 1-byte opening brace or character separator
                  (i32.add (local.get $offset) (i32.const 1))))
              ;; If the key serialization failed, or if the value does not support serialization, bail out
              (if
                (i32.or
                  (i32.ne (global.get $TRUE))
                  (i32.eqz
                    (call $Term::implements::to_json
                      (call $Term::get_type
                        (local.tee $item (call $Term::List::get_item (local.get $values) (local.get $index)))))))
                (then
                  (return (global.get $FALSE) (local.get $offset)))
                (else))
              ;; Allocate one byte for the colon separator and write the character to the output
              (call $Allocator::extend (local.get $offset) (i32.const 1))
              (i32.store8 (local.get $offset) (@char ":"))
              (local.set $offset
                (call $Term::traits::to_json
                  (local.get $item)
                  ;; The target offset is incremented to reflect the preceding 1-byte colon separator
                  (i32.add (local.get $offset) (i32.const 1))))
              ;; If the value serialization failed, bail out
              (if
                (i32.ne (global.get $TRUE))
                (then
                  (return (global.get $FALSE) (local.get $offset)))
                (else))
              ;; Allocate one byte for the comma separator or closing brace and write the character to the output
              (call $Allocator::extend (local.get $offset) (i32.const 1))
              (i32.store8
                (local.get $offset)
                ;; Choose whether to write a comma or a closing brace depending on whether this is the final item
                (select
                  (@char "}")
                  (@char ",")
                  (i32.eq
                    (local.get $index)
                    (i32.sub
                      (i32.sub (local.get $num_keys) (local.get $num_invalid_keys))
                      (i32.const 1)))))
              ;; If this was not the final valid key, continue with the next item
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (i32.sub (local.get $num_keys) (local.get $num_invalid_keys)))))))
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Return the updated offset, taking into account the final closing brace
        (i32.add (local.get $offset) (i32.const 1)))))

  (func $Term::Record::is_valid_json_key (param $key i32) (result i32)
    ;; TODO: allow JSON-encoding numeric keys
    (call $Term::String::is (local.get $key)))

  (func $Term::Record::traits::length (param $self i32) (result i32)
    (call $Term::List::traits::length (call $Term::Record::get::keys (local.get $self))))

  (func $Term::Record::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::Record::traits::size_hint (param $self i32) (result i32)
    (call $Term::Record::traits::length (local.get $self)))

  (func $Term::Record::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $index i32)
    (local $length i32)
    (if (result i32 i32 i32)
      ;; If we have iterated through all the fields, return the complete marker
      (i32.eq
        ;; Get the current iterator index from the state (initializing to zero if this is the first iteration)
        (local.tee $index
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (local.tee $length
          (call $Term::Record::traits::length (local.get $self))))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise emit a key/value entry and the incremented iterator state
        (call $Term::List::create_pair
          (call $Term::List::get_item (call $Term::Record::get::keys (local.get $self)) (local.get $index))
          (call $Term::List::get_item (call $Term::Record::get::values (local.get $self)) (local.get $index)))
        (i32.add (local.get $index) (i32.const 1))
        (global.get $NULL))))

  (func $Term::Record::traits::get (export "getRecordValue") (param $self i32) (param $key i32) (result i32)
    (local $lookup_table i32)
    (local $value i32)
    ;; Retrieve the field from struct fields or the lookup hashmap depending on whether a lookup hashmap exists
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Term::Record::get::lookup_table (local.get $self))))
      (then
        (call $Term::Record::find_value (local.get $self) (local.get $key)))
      (else
        (call $Term::Hashmap::traits::get (local.get $lookup_table) (local.get $key)))))

  (func $Term::Record::traits::has (export "hasRecordKey") (param $self i32) (param $key i32) (result i32)
    (local $lookup_table i32)
    ;; Determine field existence from struct fields or the lookup hashmap depending on whether a lookup hashmap exists
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Term::Record::get::lookup_table (local.get $self))))
      (then
        (i32.ne (global.get $NULL) (call $Term::Record::find_field_index (local.get $self) (local.get $key))))
      (else
        (call $Term::Hashmap::traits::has (local.get $lookup_table) (local.get $key)))))

  (func $Term::Record::traits::set (export "setRecordValue") (param $self i32) (param $key i32) (param $value i32) (result i32)
    (local $keys i32)
    (local $values i32)
    (local $lookup_table i32)
    (local $existing_field_index i32)
    ;; Get the updated keys and values lists for the updated record
    (if (result i32 i32)
      ;; Determine whether the given key already exists for this record
      (i32.eq
        (global.get $NULL)
        (local.tee $existing_field_index
          (if (result i32)
            ;; If there is no lookup table for this record, iterate through the fields to find a matching field index
            (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Term::Record::get::lookup_table (local.get $self))))
            (then
              (call $Term::Record::find_field_index (local.get $self) (local.get $key)))
            (else
              ;; Otherwise confirm via the lookup table whether the key exists before iterating through the fields
              (if (result i32)
                (call $Term::Hashmap::traits::has (local.get $lookup_table) (local.get $key))
                (then
                  (call $Term::Record::find_field_index (local.get $self) (local.get $key)))
                (else
                  (global.get $NULL)))))))
      (then
        ;; If an entry does not already exist for the given key, return the updated keys and values arrays
        (call $Term::List::push (call $Term::Record::get::keys (local.get $self)) (local.get $key))
        (call $Term::List::push (call $Term::Record::get::values (local.get $self)) (local.get $value)))
      (else
        (if (result i32 i32)
          ;; If the key already exists, and is already set to the provided value, return the existing record
          (call $Term::traits::equals
            (call $Term::List::get_item (local.tee $values (call $Term::Record::get::values (local.get $self))) (local.get $existing_field_index))
            (local.get $value))
          (then
            (return (local.get $self)))
          (else
            ;; Otherwise return the existing keys list with an updated values list
            (call $Term::Record::get::keys (local.get $self))
            (call $Term::List::update_index (local.get $values) (local.get $existing_field_index) (local.get $value))))))
    (local.set $values)
    (local.set $keys)
    ;; Return a new record with the updated keys and values, and an updated lookup table
    (call $Term::TermType::Record::new
      (local.get $keys)
      (local.get $values)
      (if (result i32)
        (i32.eq (global.get $NULL) (local.get $lookup_table))
        (then
          ;; Determine whether to allocate a lookup hashmap depending on the number of record fields
          (if (result i32)
            (i32.ge_u (call $Term::List::traits::length (local.get $keys)) (global.get $Term::Record::LOOKUP_TABLE_MIN_SIZE))
            (then
              (call $Term::Record::create_hashmap_lookup (local.get $keys) (local.get $values)))
            (else
              (global.get $NULL))))
        (else
          ;; Get an updated copy of the existing lookup table
          (call $Term::Hashmap::traits::set
            (local.get $lookup_table)
            (local.get $key)
            (local.get $value))))))

  (func $Term::Record::traits::keys (param $self i32) (result i32)
    (call $Term::Record::get::keys (local.get $self)))

  (func $Term::Record::traits::values (param $self i32) (result i32)
    (call $Term::Record::get::values (local.get $self)))

  (func $Term::Record::find_value (param $self i32) (param $key i32) (result i32)
    (local $field_index i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $field_index (call $Term::Record::find_field_index (local.get $self) (local.get $key))))
      (then
        ;; If a matching field was not found, return the null sentinel value
        (global.get $NULL))
      (else
        ;; Otherwise return the matched field value
        (call $Term::List::get_item (call $Term::Record::get::values (local.get $self)) (local.get $field_index)))))

  (func $Term::Record::find_field_index (param $self i32) (param $key i32) (result i32)
    (local $keys i32)
    (local $num_keys i32)
    (local $field_index i32)
    (local.set $keys (call $Term::Record::get::keys (local.get $self)))
    (local.set $num_keys (call $Term::List::traits::length (local.get $keys)))
    ;; Iterate through the keys in order until a match is located
    (local.set $field_index (i32.const 0))
    (loop $LOOP (result i32)
      (if (result i32)
        ;; If all keys have been tried unsucessfully, return the null sentinel value
        (i32.eq (local.get $field_index) (local.get $num_keys))
        (then
          (global.get $NULL))
        (else
          (if (result i32)
            ;; Check whether the current field matches the provided value
            (call $Term::traits::equals (call $Term::List::get_item (local.get $keys) (local.get $field_index)) (local.get $key))
            (then
              ;; Key matches; return current field index
              (local.get $field_index))
            (else
              ;; Try the next key
              (local.set $field_index (i32.add (local.get $field_index) (i32.const 1)))
              (br $LOOP)))))))

  (func $Term::Record::create_hashmap_lookup (param $keys i32) (param $values i32) (result i32)
    (local $num_fields i32)
    (local $hashmap i32)
    (local $field_index i32)
    (local.set $num_fields (call $Term::List::traits::length (local.get $keys)))
    ;; Allocate a new hashmap to store the lookup table
    (local.set $hashmap (call $Term::Hashmap::allocate (call $Term::Hashmap::default_capacity (call $Term::List::traits::length (local.get $keys)))))
    ;; Distribute the values into buckets
    (loop $LOOP
      (call $Term::Hashmap::insert
        (local.get $hashmap)
        (call $Term::List::get_item (local.get $keys) (local.get $field_index))
        (call $Term::List::get_item (local.get $values) (local.get $field_index)))
      (br_if $LOOP
        (i32.lt_u (local.tee $field_index (i32.add (local.get $field_index) (i32.const 1))) (local.get $num_fields))))
    ;; Instantiate the hashmap
    (call $Term::Hashmap::init (local.get $hashmap) (local.get $num_fields))))
