;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Define the minimum number of fields at which to allocate an internal hashmap lookup table
  (global $Record::LOOKUP_TABLE_MIN_SIZE i32 (i32.const 16))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Record::EMPTY (mut i32) (i32.const -1))

  (func $Record::startup
    ;; Pre-allocate the singleton instance
    (local $instance i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $instance (call $Term::new (global.get $TermType::Record) (i32.const 3)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $instance) (i32.const 0) (call $List::empty))
    (call $Term::set_field (local.get $instance) (i32.const 1) (call $List::empty))
    (call $Term::set_field (local.get $instance) (i32.const 2) (global.get $NULL))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $Record::EMPTY))

  (func $Record::new (export "createRecord") (param $keys i32) (param $values i32) (result i32)
    (local $self i32)
    (if (result i32)
      (i32.eq (call $List::traits::length (local.get $keys)) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Record::EMPTY))
      (else
        ;; Allocate a new struct of the required size and type
        (local.tee $self (call $Term::new (global.get $TermType::Record) (i32.const 3)))
        ;; Store the struct fields at the correct offsets
        (call $Term::set_field (local.get $self) (i32.const 0) (local.get $keys))
        (call $Term::set_field (local.get $self) (i32.const 1) (local.get $values))
        (call $Term::set_field (local.get $self) (i32.const 2)
          ;; Determine whether to allocate a lookup hashmap depending on the number of record fields
          (if (result i32)
            (i32.ge_u (call $List::traits::length (local.get $keys)) (global.get $Record::LOOKUP_TABLE_MIN_SIZE))
            (then
              (call $Record::create_hashmap_lookup (local.get $keys) (local.get $values)))
            (else
              (global.get $NULL))))
        ;; Instantiate the term
        (call $Term::init))))

  (func $Record::empty (result i32)
    (global.get $Record::EMPTY))

  (func $Record::is (export "isRecord") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Record) (call $Term::get_type (local.get $term))))

  (func $Record::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Record::traits::is_atomic (param $self i32) (result i32)
    (i32.and
      (call $List::traits::is_atomic (call $Record::get::keys (local.get $self)))
      (call $List::traits::is_atomic (call $Record::get::values (local.get $self)))))

  (func $Record::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Record::get::keys (export "getRecordKeys") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Record::get::values (export "getRecordValues") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $Record::get::lookup_table (export "getRecordLookupTable") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 2)))

  (func $Record::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Record::get::keys (local.get $self))
    (call $Hash::write_term)
    (call $Record::get::values (local.get $self))
    (call $Hash::write_term))

  (func $Record::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.and
      (call $Term::traits::equals (call $Record::get::keys (local.get $self)) (call $Record::get::keys (local.get $other)))
      (call $Term::traits::equals (call $Record::get::values (local.get $self)) (call $Record::get::values (local.get $other)))))

  (func $Record::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (local $keys i32)
    (local $values i32)
    (local $num_keys i32)
    (local $num_invalid_keys i32)
    (local $index i32)
    (local $key i32)
    ;; Determine the number of unserializable keys in order to subtract them from the total
    (if
      (local.tee $num_keys (call $List::get::length (local.tee $keys (call $Record::get::keys (local.get $self)))))
      (then
        (loop $LOOP
          (local.set $num_invalid_keys
            (select
              (local.get $num_invalid_keys)
              (i32.add (local.get $num_invalid_keys) (i32.const 1))
              (call $Record::is_valid_json_key (call $List::get_item (local.get $keys) (local.get $index)))))
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $num_keys)))))
      (else))
    (local.set $index (i32.const 0))
    (local.set $values (call $Record::get::values (local.get $self)))
    (if (result i32)
      ;; If the record is empty, write an empty JSON object literal
      (i32.eqz (i32.sub (local.get $num_keys) (local.get $num_invalid_keys)))
      (then
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
            (i32.eqz (call $Record::is_valid_json_key (local.tee $key (call $List::get_item (local.get $keys) (local.get $index)))))
            (then
              ;; Decrement the number of remaining invalid keys
              (local.set $num_invalid_keys (i32.sub (local.get $num_invalid_keys) (i32.const 1)))
              ;; Continue with the next item (there must be more valid keys remaining in order to get here)
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))
            (else
              ;; Write the current item to the output and store the updated offset
              (local.set $offset
                (call $Term::traits::write_json
                  (local.get $key)
                  ;; The target offset is incremented to reflect the preceding 1-byte opening brace or character separator
                  (i32.add (local.get $offset) (i32.const 1))))
              ;; Allocate one byte for the colon separator and write the character to the output
              (call $Allocator::extend (local.get $offset) (i32.const 1))
              (i32.store8 (local.get $offset) (@char ":"))
              (local.set $offset
                (call $Term::traits::write_json
                  (call $List::get_item (local.get $values) (local.get $index))
                  ;; The target offset is incremented to reflect the preceding 1-byte colon separator
                  (i32.add (local.get $offset) (i32.const 1))))
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
        ;; Return the updated offset, taking into account the final closing brace
        (i32.add (local.get $offset) (i32.const 1)))))

  (func $Record::is_valid_json_key (param $key i32) (result i32)
    ;; TODO: allow JSON-encoding numeric keys
    (call $String::is (local.get $key)))

  (func $Record::traits::length (param $self i32) (result i32)
    (call $List::traits::length (call $Record::get::keys (local.get $self))))

  (func $Record::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Record::traits::size_hint (param $self i32) (result i32)
    (call $Record::traits::length (local.get $self)))

  (func $Record::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
          (call $Record::traits::length (local.get $self))))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise emit a key/value entry and the incremented iterator state
        (call $List::create_pair
          (call $List::get_item (call $Record::get::keys (local.get $self)) (local.get $index))
          (call $List::get_item (call $Record::get::values (local.get $self)) (local.get $index)))
        (i32.add (local.get $index) (i32.const 1))
        (global.get $NULL))))

  (func $Record::traits::get (export "getRecordValue") (param $self i32) (param $key i32) (result i32)
    (local $lookup_table i32)
    (local $value i32)
    ;; Retrieve the field from struct fields or the lookup hashmap depending on whether a lookup hashmap exists
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Record::get::lookup_table (local.get $self))))
      (then
        (call $Record::find_value (local.get $self) (local.get $key)))
      (else
        (call $Hashmap::traits::get (local.get $lookup_table) (local.get $key)))))

  (func $Record::traits::has (export "hasRecordKey") (param $self i32) (param $key i32) (result i32)
    (local $lookup_table i32)
    ;; Determine field existence from struct fields or the lookup hashmap depending on whether a lookup hashmap exists
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Record::get::lookup_table (local.get $self))))
      (then
        (i32.ne (global.get $NULL) (call $Record::find_field_index (local.get $self) (local.get $key))))
      (else
        (call $Hashmap::traits::has (local.get $lookup_table) (local.get $key)))))

  (func $Record::traits::set (export "setRecordValue") (param $self i32) (param $key i32) (param $value i32) (result i32)
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
            (i32.eq (global.get $NULL) (local.tee $lookup_table (call $Record::get::lookup_table (local.get $self))))
            (then
              (call $Record::find_field_index (local.get $self) (local.get $key)))
            (else
              ;; Otherwise confirm via the lookup table whether the key exists before iterating through the fields
              (if (result i32)
                (call $Hashmap::traits::has (local.get $lookup_table) (local.get $key))
                (then
                  (call $Record::find_field_index (local.get $self) (local.get $key)))
                (else
                  (global.get $NULL)))))))
      (then
        ;; If an entry does not already exist for the given key, return the updated keys and values arrays
        (call $List::push (call $Record::get::keys (local.get $self)) (local.get $key))
        (call $List::push (call $Record::get::values (local.get $self)) (local.get $value)))
      (else
        (if (result i32 i32)
          ;; If the key already exists, and is already set to the provided value, return the existing record
          (call $Term::traits::equals
            (call $List::get_item (local.tee $values (call $Record::get::values (local.get $self))) (local.get $existing_field_index))
            (local.get $value))
          (then
            (return (local.get $self)))
          (else
            ;; Otherwise return the existing keys list with an updated values list
            (call $Record::get::keys (local.get $self))
            (call $List::update_index (local.get $values) (local.get $existing_field_index) (local.get $value))))))
    (local.set $values)
    (local.set $keys)
    ;; Return a new record with the updated keys and values, and an updated lookup table if one already existed
    ;; FIXME: Create a lookup table if adding this record entry caused the number of fields to reach the minimum lookup table size
    (local.tee $self (call $Term::new (global.get $TermType::Record) (i32.const 3)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $keys))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $values))
    (call $Term::set_field
      (local.get $self)
      (i32.const 2)
      (if (result i32)
        (i32.eq (global.get $NULL) (local.get $lookup_table))
        (then
          (global.get $NULL))
        (else
          (call $Hashmap::traits::set (local.get $lookup_table) (local.get $key) (local.get $value)))))
    (call $Term::init))

  (func $Record::traits::keys (param $self i32) (result i32)
    (call $Record::get::keys (local.get $self)))

  (func $Record::traits::values (param $self i32) (result i32)
    (call $Record::get::values (local.get $self)))

  (func $Record::find_value (param $self i32) (param $key i32) (result i32)
    (local $field_index i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $field_index (call $Record::find_field_index (local.get $self) (local.get $key))))
      (then
        ;; If a matching field was not found, return the null sentinel value
        (global.get $NULL))
      (else
        ;; Otherwise return the matched field value
        (call $List::get_item (call $Record::get::values (local.get $self)) (local.get $field_index)))))

  (func $Record::find_field_index (param $self i32) (param $key i32) (result i32)
    (local $keys i32)
    (local $num_keys i32)
    (local $field_index i32)
    (local.set $keys (call $Record::get::keys (local.get $self)))
    (local.set $num_keys (call $List::traits::length (local.get $keys)))
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
            (call $Term::traits::equals (call $List::get_item (local.get $keys) (local.get $field_index)) (local.get $key))
            (then
              ;; Key matches; return current field index
              (local.get $field_index))
            (else
              ;; Try the next key
              (local.set $field_index (i32.add (local.get $field_index) (i32.const 1)))
              (br $LOOP)))))))

  (func $Record::create_hashmap_lookup (param $keys i32) (param $values i32) (result i32)
    (local $num_fields i32)
    (local $hashmap i32)
    (local $field_index i32)
    (local.set $num_fields (call $List::traits::length (local.get $keys)))
    ;; Allocate a new hashmap to store the lookup table
    (local.set $hashmap (call $Hashmap::allocate (call $Hashmap::default_capacity (call $List::traits::length (local.get $keys)))))
    ;; Distribute the values into buckets
    (loop $LOOP
      (call $Hashmap::insert
        (local.get $hashmap)
        (call $List::get_item (local.get $keys) (local.get $field_index))
        (call $List::get_item (local.get $values) (local.get $field_index)))
      (br_if $LOOP
        (i32.lt_u (local.tee $field_index (i32.add (local.get $field_index) (i32.const 1))) (local.get $num_fields))))
    ;; Instantiate the hashmap
    (call $Hashmap::init (local.get $hashmap) (local.get $num_fields))))
