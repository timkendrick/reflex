;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Minimum list capacity when allocating non-zero-length lists of unknown size
  (global $List::MIN_UNSIZED_LIST_CAPACITY i32 (i32.const 8))
  (global $List::NUM_HEADER_FIELDS i32 (i32.const 1))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $List::EMPTY (mut i32) (i32.const -1))

  (func $List::startup
    ;; Pre-allocate the singleton instance
    (local $instance i32)
    ;; Allocate a new struct of the required size and type (one field for the list length; no additional items)
    (local.tee $instance (call $Term::new (global.get $TermType::List) (global.get $List::NUM_HEADER_FIELDS)))
    ;; Store the list length as the first field
    (call $Term::set_field (local.get $instance) (i32.const 0) (i32.const 0))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $List::EMPTY))

  (func $List::allocate (export "allocateList") (param $capacity i32) (result i32)
    ;; Allocates a new List term with the given capacity, allowing items to be copied directly into the allocated slots.
    ;; The list must be instantiated before it can be used.
    ;; TODO: Investigate unique interned lists
    (local $self i32)
    (if (result i32)
      (i32.eq (local.get $capacity) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $List::EMPTY))
      (else
        ;; Allocate a new struct of the required size and type (one field for the list length; plus one field per item)
        (local.tee $self
          (call $Term::new
            (global.get $TermType::List)
            (i32.add (global.get $List::NUM_HEADER_FIELDS) (local.get $capacity)))))))

  (func $List::init (export "initList") (param $self i32) (param $length i32) (result i32)
    ;; This assumes the given list has already been allocated and filled with items
    ;; Store the list length
    (call $List::set::length (local.get $self) (local.get $length))
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $List::allocate_unsized (result i32)
    ;; This creates a new list of unknown size.
    ;; The list MUST be filled and initialized before any further allocations can take place.
    (call $List::empty))

  (func $List::grow_unsized (param $self i32) (param $value i32) (result i32)
    ;; This extends an existing unsized list into unallocated free space.
    ;; This will panic if the list is not the most recent heap object to have been allocated.
    (local $length i32)
    (if (result i32)
      ;; If this is the first list item, allocate a new list
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        ;; Allocate a new struct of the required size and type (one field for the list length; one for the list item)
        (local.tee $self (call $Term::new (global.get $TermType::List) (i32.add (global.get $List::NUM_HEADER_FIELDS) (i32.const 1))))
        (call $List::set::length (local.get $self) (i32.const 1))
        (call $List::set_item (local.get $self) (i32.const 0) (local.get $value)))
      (else
        ;; Otherwise allocate space for another list item, and assert that the allocated slot is contiguous with the existing items
        (if (result i32)
          (i32.ne
            (call $Allocator::allocate (i32.const 4))
            (call $List::get_item_pointer
              (local.get $self)
              (local.get $length)))
          (then
            ;; Panic if the allocated slot is not contiguous. This will happen if there were intermediate allocations
            ;; unrelated to the list building since the most recent item was added
            (unreachable))
          (else
            ;; Add the item to the list and update the list length
            (call $List::set_item (local.get $self) (local.get $length) (local.get $value))
            (call $List::set::length (local.get $self) (local.tee $length (i32.add (local.get $length) (i32.const 1))))
            ;; Update the term size with the correct capacity
            (call $Term::set_num_fields (local.get $self) (i32.add (global.get $List::NUM_HEADER_FIELDS) (local.get $length)))
            (local.get $self))))))

  (func $List::append_unsized (param $self i32) (param $value i32) (result i32)
    ;; This extends an existing unsized list into unallocated free space if it is the most recent heap object to have been allocated,
    ;; or reallocates a larger list when necessary if there have been more recent heap allocations
    (local $length i32)
    (if (result i32)
      ;; If this is the first list item, allocate a new list
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        ;; Allocate a new struct of the required size and type (one field for the list length; one for the list item)
        (local.tee $self (call $Term::new (global.get $TermType::List) (i32.add (global.get $List::NUM_HEADER_FIELDS) (global.get $List::MIN_UNSIZED_LIST_CAPACITY))))
        (call $List::set_item (local.get $self) (i32.const 0) (local.get $value))
        (call $List::set::length (local.get $self) (i32.const 1)))
      (else
        ;; Otherwise determine whether the list has space for another list item, reallocating to a larger slot if necessary
        (if
          (i32.eq (local.get $length) (call $List::get_capacity (local.get $self)))
          (then
            ;; Reallocate the list with double the capacity
            (local.set $self (call $List::reallocate (local.get $self) (i32.mul (local.get $length) (i32.const 2)))))
          (else))
        ;; Add the item to the list and update the list length
        (call $List::set_item (local.get $self) (local.get $length) (local.get $value))
        (call $List::set::length (local.get $self) (local.tee $length (i32.add (local.get $length) (i32.const 1))))
        (local.get $self))))

  (func $List::init_unsized (param $self i32) (result i32)
    (local $length i32)
    (if (result i32)
      ;; If there were no items added to the list, return the empty list
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        (local.get $self))
      (else
        ;; Otherwise instantiate the term
        (call $List::init (local.get $self) (local.get $length)))))

  (func $List::empty (export "createEmptyList") (result i32)
    ;; Allocate a new list of the required length
    ;; (this will return the pre-allocated empty list singleton)
    (call $List::allocate (i32.const 0)))

  (func $List::of (export "createUnitList") (param $value i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required length
    (local.tee $self (call $List::allocate (i32.const 1)))
    ;; Store the list items at the correct offsets
    (call $List::set_item (local.get $self) (i32.const 0) (local.get $value))
    ;; Instantiate the list term
    (call $List::init (i32.const 1)))

  (func $List::create_pair (export "createPair") (param $left i32) (param $right i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required length
    (local.tee $self (call $List::allocate (i32.const 2)))
    ;; Store the list items at the correct offsets
    (call $List::set_item (local.get $self) (i32.const 0) (local.get $left))
    (call $List::set_item (local.get $self) (i32.const 1) (local.get $right))
    ;; Instantiate the list term
    (call $List::init (i32.const 2)))

  (func $List::create_triple (export "createTriple") (param $first i32) (param $second i32) (param $third i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required length
    (local.tee $self (call $List::allocate (i32.const 3)))
    ;; Store the list items at the correct offsets
    (call $List::set_item (local.get $self) (i32.const 0) (local.get $first))
    (call $List::set_item (local.get $self) (i32.const 1) (local.get $second))
    (call $List::set_item (local.get $self) (i32.const 2) (local.get $third))
    ;; Instantiate the list term
    (call $List::init (i32.const 3)))

  (func $List::is (export "isList") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::List) (call $Term::get_type (local.get $term))))

  (func $List::get::length (export "getListLength") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $List::set::length (param $self i32) (param $value i32)
    ;; Update the struct field value at the correct offset
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value)))

  (func $List::get::items (export "getListItems") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field_pointer (local.get $self) (i32.const 1)))

  (func $List::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $List::traits::is_atomic (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return true
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is not atomic, return false
            (i32.eqz (call $Term::traits::is_atomic (call $List::get_item (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no non-atomic items were encountered in the entire list, return true
        (global.get $TRUE))))

  (func $List::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $List::traits::hash (param $self i32) (param $state i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local.get $state)
    ;; Hash the list length
    (local.tee $length (call $List::get::length (local.get $self)))
    (local.set $state (call $Hash::write_i32))
    ;; Hash the list items
    (if (result i32)
      (i32.eq (local.get $length) (i32.const 0))
      ;; If the list is empty, nothing more to do
      (then
        (local.get $state))
      (else
        ;; Hash each of the list items
        (loop $LOOP (result i32)
          (local.get $state)
          (call $List::get_item (local.get $self) (local.get $index))
          (local.set $state (call $Hash::write_term))
          ;; If this was the final item return the hash, otherwise continue with the next item
          (if (result i32)
            (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))
            (then
              (local.get $state))
            (else
              (br $LOOP)))))))

  (func $List::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    ;; (this makes the assumption that lists with the same length and hash are almost certainly identical)
    (i32.and
      (i32.eq (call $List::get::length (local.get $self)) (call $List::get::length (local.get $other)))
      (i32.eq (call $Term::get_hash (local.get $self)) (call $Term::get_hash (local.get $other)))))

  (func $List::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (local $index i32)
    (local $length i32)
    (if (result i32)
      ;; If the list is empty, write an empty JSON array literal
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        ;; Allocate two bytes for opening and closing braces and write the characters to the output
        (call $Allocator::extend (local.get $offset) (i32.const 2))
        (i32.store8 offset=0 (local.get $offset) (@char "["))
        (i32.store8 offset=1 (local.get $offset) (@char "]"))
        ;; Return the number of bytes written
        (i32.add (local.get $offset) (i32.const 2)))
      (else
        ;; Allocate one byte for the opening brace and write the character to the output
        (call $Allocator::extend (local.get $offset) (i32.const 1))
        (i32.store8 (local.get $offset) (@char "["))
        ;; Iterate through the list items
        (loop $LOOP
          ;; Write the current item to the output and store the updated offset
          (local.set $offset
            (call $Term::traits::write_json
              (call $List::get_item (local.get $self) (local.get $index))
              ;; The target offset is incremented to reflect the preceding 1-byte opening brace or character separator
              (i32.add (local.get $offset) (i32.const 1))))
          ;; Allocate one byte for the comma separator or closing brace and write the character to the output
          (call $Allocator::extend (local.get $offset) (i32.const 1))
          (i32.store8
            (local.get $offset)
            ;; Choose whether to write a comma or a closing brace depending on whether this is the final item
            (select
              (@char "]")
              (@char ",")
              (i32.eq (local.get $index) (i32.sub (local.get $length) (i32.const 1)))))
          ;; If this was not the final item, continue with the next item
          (br_if $LOOP
            (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))
        ;; Return the updated offset, taking into account the final closing brace
        (i32.add (local.get $offset) (i32.const 1)))))

  (func $List::traits::length (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $List::get::length (local.get $self)))

  (func $List::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $List::traits::size_hint (param $self i32) (result i32)
    (call $List::traits::length (local.get $self)))

  (func $List::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; If we have iterated through all the items, return the complete marker
      (i32.eq
        ;; Get the current iterator index from the state (initializing to zero if this is the first iteration)
        (local.tee $iterator_state
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (call $List::get::length (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise emit the current item and the incremented iterator state
        (call $List::get_item (local.get $self) (local.get $iterator_state))
        (i32.add (local.get $iterator_state) (i32.const 1))
        (global.get $NULL))))

  (func $List::traits::get (param $self i32) (param $key i32) (result i32)
    (local $index i32)
    (if (result i32)
      ;; If the key is not an integer, return the null sentinel value
      (i32.eqz (call $Int::is (local.get $key)))
      (then
        (global.get $NULL))
      (else
        (local.set $index (call $Int::get::value (local.get $key)))
        (if (result i32)
          ;; Determine whether the index is within the list bounds
          (i32.and
            (i32.ge_s (local.tee $index (call $Int::get::value (local.get $key))) (i32.const 0))
            (i32.lt_u (local.get $index) (call $List::get::length (local.get $self))))
          (then
            ;; If the index is within the list bounds, etrieve the corresponding list item
            (call $List::get_item (local.get $self) (local.get $index)))
          (else
            ;; Otherwise return the null sentinel value
            (global.get $NULL))))))

  (func $List::traits::has (param $self i32) (param $key i32) (result i32)
    (local $index i32)
    (if (result i32)
      ;; If the key is not an integer, return false
      (i32.eqz (call $Int::is (local.get $key)))
      (then
        (global.get $FALSE))
      (else
        ;; Determine whether the index is within the list bounds
        (i32.and
          (i32.ge_s (local.tee $index (call $Int::get::value (local.get $key))) (i32.const 0))
          (i32.lt_u (local.get $index) (call $List::get::length (local.get $self)))))))

  (func $List::traits::keys (param $self i32) (result i32)
    (call $RangeIterator::new (i32.const 0) (call $List::get::length (local.get $self))))

  (func $List::traits::values (param $self i32) (result i32)
    (call $List::traits::iterate (local.get $self)))

  (func $List::traits::union (param $self i32) (param $other i32) (result i32)
    (local $result i32)
    (local $length_self i32)
    (local $length_other i32)
    (local.set $length_self (call $List::get::length (local.get $self)))
    (local.set $length_other (call $List::get::length (local.get $other)))
    (local.tee $result (call $List::allocate (i32.add (local.get $length_self) (local.get $length_other))))
    (memory.copy
      (call $List::get_item_pointer (local.get $result) (i32.const 0))
      (call $List::get_item_pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $length_self) (i32.const 4)))
    (memory.copy
      (call $List::get_item_pointer (local.get $result) (local.get $length_self))
      (call $List::get_item_pointer (local.get $other) (i32.const 0))
      (i32.mul (local.get $length_other) (i32.const 4)))
    ;; Instantiate the list term
    (call $List::init (i32.add (local.get $length_self) (local.get $length_other))))

  (func $List::get_capacity (export "getListCapacity") (param $self i32) (result i32)
    ;; Determine the list capacity based on the allocated term size
    (i32.sub (call $Term::get_num_fields (local.get $self)) (global.get $List::NUM_HEADER_FIELDS)))

  (func $List::get_item_pointer (param $list i32) (param $index i32) (result i32)
    (i32.add
      (call $List::get::items (local.get $list))
      (i32.mul (local.get $index) (i32.const 4))))

  (func $List::get_item (export "getListItem") (param $self i32) (param $index i32) (result i32)
    (i32.load (call $List::get_item_pointer (local.get $self) (local.get $index))))

  (func $List::set_item (param $self i32) (param $index i32) (param $value i32)
    ;; Note that this does not bounds-check the list capacity or update the hash, it merely inserts an entry into an already-allocated list
    (i32.store (call $List::get_item_pointer (local.get $self) (local.get $index)) (local.get $value)))

  (func $List::update_index (param $self i32) (param $index i32) (param $value i32) (result i32)
    ;; Return a clone of the given list, with the specified index set to the given value
    ;; This assumes the given index is within the list bounds
    ;; Create a clone of the original list
    (local.tee $self (call $Term::traits::clone (local.get $self)))
    ;; Update the specified index with the specified value
    (call $List::set_item (local.get $self) (local.get $index) (local.get $value))
    ;; Instantiate the term
    (call $Term::init))

  (func $List::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a list, return the existing instance
      (call $List::is (local.get $iterator))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the list items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (local.tee $length (call $Term::traits::size_hint (local.get $iterator))) (global.get $NULL))
          (then
            (call $List::collect_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $List::collect_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $List::collect_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty list
      (i32.eqz (local.get $length))
      (then
        (call $List::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new list to hold the results and fill it by consuming each iterator item in turn
        (local.tee $instance (call $List::allocate (local.get $length)))
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
              ;; Otherwise store the item in the results list and continue with the next item
              (call $List::set_item (local.get $instance) (local.get $index) (local.get $item))
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))))
        ;; Initialize the list term
        (call $List::init (local.get $index))
        (local.get $dependencies))))

  (func $List::collect_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; We cannot know in advance the correct size of list to allocate, so we start off with the empty list, then
    ;; allocate a series of lists of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    ;; Start off with the empty list to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $List::empty))
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
          ;; Otherwise store the item in the results list,
          ;; reallocating a new list if the existing results list has reached its capacity
          (if
            ;; If the current list is full, reallocate a new list with larger capacity
            (i32.eq (local.get $index) (local.get $capacity))
            (then
              ;; Assign the list length to ensure that the reallocation copies all the items collected so far
              (call $List::set::length (local.get $instance) (local.get $index))
              ;; Reallocate the list to a new location with double the capacity
              (local.set $instance
                (call $List::reallocate
                  (local.get $instance)
                  (local.tee $capacity
                    (select
                      ;; If this is the first non-empty allocation, create a list of a predetermined capacity
                      (global.get $List::MIN_UNSIZED_LIST_CAPACITY)
                      ;; Otherwise create a new list with double the existing capacity
                      ;; (this ensures amortized list allocations as the number of items increases)
                      (i32.mul (local.get $capacity) (i32.const 2))
                      (i32.eqz (local.get $capacity)))))))
            (else))
          ;; Store the item in the results list and continue with the next item
          (call $List::set_item (local.get $instance) (local.get $index) (local.get $item))
          (local.set $index (i32.add (local.get $index) (i32.const 1)))
          (br $LOOP))))
    (if (result i32 i32)
      ;; If the source iterator did not produce any items, return the empty results list as-is
      (i32.eqz (local.get $index))
      (then
        (local.get $instance)
        (local.get $dependencies))
      (else
        ;; Otherwise initialize the list term
        (call $List::init (local.get $instance) (local.get $index))
        (local.get $dependencies))))

  (func $List::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a list composed solely of static items, return the existing instance
      (if (result i32)
        (call $List::is (local.get $iterator))
        (then
          (i32.eqz (call $List::has_dynamic_items (local.get $iterator))))
        (else
          (global.get $FALSE)))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the list items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.tee $length (call $Term::traits::size_hint (local.get $iterator))))
          (then
            (call $List::collect_strict_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $List::collect_strict_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $List::collect_strict_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty list
      (i32.eqz (local.get $length))
      (then
        (call $List::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new list to hold the results and fill it by consuming each iterator item in turn
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $signal (global.get $NULL))
        (local.set $instance (call $List::allocate (local.get $length)))
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
              ;; Resolve the list item
              (call $Term::traits::evaluate (local.get $item) (local.get $state))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $item)
              (if
                ;; If the current item is a signal, or a signal has already been encountered,
                ;; update the combined signal and continue with the next item
                (i32.ne
                  (global.get $NULL)
                  (local.tee $signal
                    (call $Signal::traits::union
                      (local.get $signal)
                      (select
                        (local.get $item)
                        (global.get $NULL)
                        (call $Signal::is (local.get $item))))))
                (then
                  ;; Continue with the next item
                  (local.set $index (i32.add (local.get $index) (i32.const 1)))
                  (br $LOOP))
                (else
                  ;; Otherwise store the item in the results list
                  (call $List::set_item (local.get $instance) (local.get $index) (local.get $item))
                  ;; Continue with the next item
                  (local.set $index (i32.add (local.get $index) (i32.const 1)))
                  (br $LOOP))))))
        (if (result i32 i32)
          ;; If a signal was encountered during the iteration, return the combined signal
          (i32.ne (global.get $NULL) (local.get $signal))
          (then
            (local.get $signal)
            (local.get $dependencies))
          (else
            ;; Otherwise initialize the results list
            (call $List::init (local.get $instance) (local.get $index))
            (local.get $dependencies))))))

  (func $List::collect_strict_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; We cannot know in advance the correct size of list to allocate, so we start off with the empty list, then
    ;; allocate a series of lists of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    ;; Start off with the empty list to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $List::empty))
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
          ;; Resolve the list item
          (call $Term::traits::evaluate (local.get $item) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $item)
          ;; If the current item is a signal, or a signal has already been encountered,
          ;; update the combined signal and continue with the next item
          (if
            (i32.ne
              (global.get $NULL)
              (local.tee $signal
                (call $Signal::traits::union
                  (local.get $signal)
                  (select
                    (local.get $item)
                    (global.get $NULL)
                    (call $Signal::is (local.get $item))))))
            (then
              ;; Continue with the next item
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))
            (else
              ;; Otherwise store the item in the results list,
              ;; reallocating a new list if the existing results list has reached its capacity
              (if
                (i32.eq (local.get $index) (local.get $capacity))
                (then
                  ;; Assign the list length to ensure that the reallocation copies all the items collected so far
                  (call $List::set::length (local.get $instance) (local.get $index))
                  ;; Reallocate the list to a new location with double the capacity
                  (local.set $instance
                    (call $List::reallocate
                      (local.get $instance)
                      (local.tee $capacity
                        (select
                          ;; If this is the first non-empty allocation, create a list of a predetermined capacity
                          (global.get $List::MIN_UNSIZED_LIST_CAPACITY)
                          ;; Otherwise create a new list with double the existing capacity
                          ;; (this ensures amortized list allocations as the number of items increases)
                          (i32.mul (local.get $capacity) (i32.const 2))
                          (i32.eqz (local.get $capacity)))))))
                (else))
              (call $List::set_item (local.get $instance) (local.get $index) (local.get $item))
              ;; Continue with the next item
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))))))
    (if (result i32 i32)
      ;; If a signal was encountered during the iteration, return the combined signal
      (i32.ne (global.get $NULL) (local.get $signal))
      (then
        (local.get $signal)
        (local.get $dependencies))
      (else
        ;; Otherwise if the source iterator did not produce any items, return the empty results list as-is
        (if (result i32 i32)
          (i32.eqz (local.get $index))
          (then
            (local.get $instance)
            (local.get $dependencies))
          (else
            ;; Otherwise initialize the results list
            (call $List::init (local.get $instance) (local.get $index))
            (local.get $dependencies))))))

  (func $List::reallocate (param $self i32) (param $capacity i32) (result i32)
    (local $instance i32)
    (if (result i32)
      ;; If the list already has sufficient capacity, return it as-is
      (i32.ge_u (call $List::get_capacity (local.get $self)) (local.get $capacity))
      (then
        (local.get $self))
      (else
        ;; Otherwise allocate a new list with the given capacity
        (local.tee $instance (call $List::allocate (local.get $capacity)))
        ;; If the source list contains any items, copy them across to the new list
        (if
          (i32.eqz (call $List::get::length (local.get $self)))
          (then)
          (else
            (memory.copy
              (call $List::get_item_pointer (local.get $instance) (i32.const 0))
              (call $List::get_item_pointer (local.get $self) (i32.const 0))
              (i32.mul (call $List::get::length (local.get $self)) (i32.const 4)))))
        ;; Rewrite the source list as a redirect pointer term
        ;; (this is to avoid breaking any existing pointers to the original list address)
        (call $Term::redirect (local.get $self) (local.get $instance)))))

  (func $List::has_dynamic_items (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return false
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        (global.get $FALSE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is dynamic, return true
            (i32.eqz (call $Term::traits::is_static (call $List::get_item (local.get $self) (local.get $index))))
            (then
              (return (global.get $TRUE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no dynamic items were encountered in the entire list, return false
        (global.get $FALSE))))

  (func $List::find_index (param $self i32) (param $value i32) (result i32)
    (local $num_items i32)
    (local $item_index i32)
    (local.set $num_items (call $List::get::length (local.get $self)))
    ;; Iterate through the items in order until a match is located
    (local.set $item_index (i32.const 0))
    (loop $LOOP (result i32)
      (if (result i32)
        ;; If all items have been tried unsucessfully, return the null sentinel value
        (i32.eq (local.get $item_index) (local.get $num_items))
        (then
          (global.get $NULL))
        (else
          (if (result i32)
            ;; Check whether the current field matches the provided value
            (call $Term::traits::equals (call $List::get_item (local.get $self) (local.get $item_index)) (local.get $value))
            (then
              ;; Item matches; return current index
              (local.get $item_index))
            (else
              ;; Try the next item
              (local.set $item_index (i32.add (local.get $item_index) (i32.const 1)))
              (br $LOOP)))))))

  (func $List::push (param $self i32) (param $value i32) (result i32)
    (local $instance i32)
    (local $existing_length i32)
    ;; Allocate a new list with the correct length
    (local.tee $instance (call $List::allocate (i32.add (local.tee $existing_length (call $List::get::length (local.get $self))) (i32.const 1))))
    ;; Copy the existing values into the new list
    (memory.copy
      (call $List::get_item_pointer (local.get $instance) (i32.const 0))
      (call $List::get_item_pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $existing_length) (i32.const 4)))
    ;; Add the provided value to the new list
    (call $List::set_item (local.get $instance) (local.get $existing_length) (local.get $value))
    ;; Instantiate the new list
    (call $List::init (i32.add (local.get $existing_length) (i32.const 1))))

  (func $List::push_front (param $self i32) (param $value i32) (result i32)
    (local $instance i32)
    (local $existing_length i32)
    ;; Allocate a new list with the correct length
    (local.tee $instance (call $List::allocate (i32.add (local.tee $existing_length (call $List::get::length (local.get $self))) (i32.const 1))))
    ;; Copy the existing values into the new list
    (memory.copy
      (call $List::get_item_pointer (local.get $instance) (i32.const 1))
      (call $List::get_item_pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $existing_length) (i32.const 4)))
    ;; Add the provided value to the new list
    (call $List::set_item (local.get $instance) (i32.const 0) (local.get $value))
    ;; Instantiate the new list
    (call $List::init (i32.add (local.get $existing_length) (i32.const 1))))

  (func $List::slice (param $self i32) (param $offset i32) (param $length i32) (result i32)
    (local $instance i32)
    (local $source_length i32)
    (if (result i32)
      ;; If the specified region encompasses the whole list, return the unmodified list
      (i32.and
        (i32.eqz (local.get $offset))
        (i32.ge_u (local.get $length) (local.tee $source_length (call $List::get::length (local.get $self)))))
      (then
        (local.get $self))
      (else
        ;; Otherwise if the specified offset is beyond the end of the list, or the length is zero, return the empty list
        (if (result i32)
          (i32.or
            (i32.ge_u (local.get $offset) (local.get $source_length))
            (i32.eqz (local.get $length)))
          (then
            (call $List::empty))
          (else
            ;; Otherwise allocate a new list of the correct length
            (local.tee $instance
              (call $List::allocate
                (local.tee $length
                  (call $Utils::i32::min_u
                    (local.get $length)
                    (i32.sub (local.get $source_length) (local.get $offset))))))
            ;; Copy the existing items into the new list
            (memory.copy
              (call $List::get_item_pointer (local.get $instance) (i32.const 0))
              (call $List::get_item_pointer (local.get $self) (local.get $offset))
              (i32.mul (local.get $length) (i32.const 4)))
            ;; Instantiate the new list
            (call $List::init (local.get $length)))))))

  (func $List::is_typed_list (param $self i32) (param $type i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return true
      (i32.eqz (local.tee $length (call $List::get::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is not of the given type, return false
            (i32.ne (local.get $type) (call $Term::get_type (call $List::get_item (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no items not of the given type were encountered in the entire list, return true
        (global.get $TRUE))))

  (func $List::reverse_in_place (param $self i32)
    (local $length i32)
    (local $index i32)
    (local $right_index i32)
    (local $left_value i32)
    (local $right_value i32)
    (if
      ;; If the list is empty, nothing to do
      (i32.lt_u (local.tee $length (call $List::get::length (local.get $self))) (i32.const 2))
      (then)
      (else
        ;; Otherwise iterate through each item in the left hand half of the list,
        ;; swapping it with the corresponding item in the right hand half of the list
        (loop $LOOP
          ;; Determine the index of the corresponding item in the right-hand half of the list
          (local.set $right_index (i32.sub (i32.sub (local.get $length) (i32.const 1)) (local.get $index)))
          ;; Swap the left-hand and right-hand items
          (local.set $left_value (call $List::get_item (local.get $self) (local.get $index)))
          (local.set $right_value (call $List::get_item (local.get $self) (local.get $right_index)))
          (call $List::set_item (local.get $self) (local.get $index) (local.get $right_value))
          (call $List::set_item (local.get $self) (local.get $right_index) (local.get $left_value))
          ;; If we have not yet reached the middle of the list, continue with the next item
          (br_if $LOOP
            (i32.lt_u
              (local.tee $index (i32.add (local.get $index) (i32.const 1)))
              (i32.div_u (local.get $length) (i32.const 2))))))))

  (func $List::allocate_partition_list (param $length i32) (result i32)
    ;; In order to minimize allocations, we allocate a single list that is large enough to store the items from both
    ;; partitions, and then once all items have been added we subdivide the list into two separate sub-lists.
    ;; Seeing as we cannot know in advance how large each partition needs to be, items will be added at either end
    ;; depending on which partition they belong to. The list will be split into two sub-lists when it is initialized.
    (local $instance i32)
    (local $capacity i32)
    (local.tee $instance
      (call $List::allocate
        (local.tee $capacity
          (i32.add
            ;; Seeing as we are creating a single contiguous list that will later be split into two lists, we need to
            ;; allocate additional space for the right-hand partition's list header (this will be added retrospectively)
            (i32.add (global.get $Term::NUM_HEADER_FIELDS) (global.get $List::NUM_HEADER_FIELDS))
            (local.get $length)))))
    ;; While building the lists, the list length field will be used to track the current offsets of the two partitions.
    ;; Each offset value indicates the field index of the next available slot for the given partition.
    ;; The left partition is filled from the left-hand side; the right partition is filled from the right-hand side.
    ;; This means that the left-hand offset counts up from zero, while the right-hand offset counts down from (len - 1).
    ;; Repurposing the list length field requires that we fit two offsets into a single field; this is done by splitting
    ;; the 32-bit list length field into two 16-bit integers representing the left and right partition offsets.
    ;; Note that this means that neither partition can exceed 2^16 items in length.
    ;; This field will be overridden by the final length of the left-hand partition when the list is initialized.
    (call $List::set_partition_list_offset_left (local.get $instance) (i32.const 0))
    (call $List::set_partition_list_offset_right (local.get $instance) (i32.sub (local.get $capacity) (i32.const 1))))

  (func $List::get_partition_list_offset_left_pointer (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $List::get_partition_list_offset_right_pointer (param $self i32) (result i32)
    (i32.add (call $List::get_partition_list_offset_left_pointer (local.get $self)) (i32.const 2)))

  (func $List::get_partition_list_offset_left (param $self i32) (result i32)
    (i32.load16_u (call $List::get_partition_list_offset_left_pointer (local.get $self))))

  (func $List::set_partition_list_offset_left (param $self i32) (param $value i32)
    (i32.store16
      (call $List::get_partition_list_offset_left_pointer (local.get $self))
      (i32.and (local.get $value) (i32.const 0x0000FFFF))))

  (func $List::get_partition_list_offset_right (param $self i32) (result i32)
    (i32.load16_u (call $List::get_partition_list_offset_right_pointer (local.get $self))))

  (func $List::set_partition_list_offset_right (param $self i32) (param $value i32)
    (i32.store16
      (call $List::get_partition_list_offset_right_pointer (local.get $self))
      (i32.and (local.get $value) (i32.const 0x0000FFFF))))

  (func $List::insert_partition_list_item (param $self i32) (param $partition i32) (param $value i32)
    (if
      (i32.eqz (local.get $partition))
      (then
        (call $List::insert_partition_list_item_left (local.get $self) (local.get $value)))
      (else
        (call $List::insert_partition_list_item_right (local.get $self) (local.get $value)))))

  (func $List::insert_partition_list_item_left (param $self i32) (param $value i32)
    (local $offset i32)
    ;; Insert the list item at the current left-hand partition offset
    (i32.store
      (call $List::get_item_pointer
        (local.get $self)
        (local.tee $offset (call $List::get_partition_list_offset_left (local.get $self))))
      (local.get $value))
    ;; Increment the left-hand partition offset field
    (call $List::set_partition_list_offset_left (local.get $self) (i32.add (local.get $offset) (i32.const 1))))

  (func $List::insert_partition_list_item_right (param $self i32) (param $value i32)
    (local $offset i32)
    ;; Insert the list item at the current right-hand partition offset
    (i32.store
      (call $List::get_item_pointer
        (local.get $self)
        (local.tee $offset (call $List::get_partition_list_offset_right (local.get $self))))
      (local.get $value))
    ;; Decrement the right-hand partition offset field
    (call $List::set_partition_list_offset_right (local.get $self) (i32.sub (local.get $offset) (i32.const 1))))

  (func $List::init_partition_list_unordered (param $self i32) (result i32 i32)
    ;; This splits the list at the correct partition boundary, leaving the items in the positions they were added
    ;; (i.e. the left-hand partition will retain the original order, but the right-hand partition will be reversed)
    (local $right_offset i32)
    (local $left_length i32)
    (local $right_length i32)
    ;; Determine the length of each partition based on the stored partition offsets
    (local.set $left_length (call $List::get_partition_list_offset_left (local.get $self)))
    (local.set $right_length
      (i32.sub
        (i32.sub (call $List::get_capacity (local.get $self)) (i32.const 1))
        (local.tee $right_offset (call $List::get_partition_list_offset_right (local.get $self)))))
    ;; Shrink the left-hand list capacity to the correct size to make space for the right-hand list
    (call $Term::set_num_fields
      (local.get $self)
      (i32.sub
        (call $Term::get_num_fields (local.get $self))
        (i32.add
          (i32.add (global.get $Term::NUM_HEADER_FIELDS) (global.get $List::NUM_HEADER_FIELDS))
          (local.get $right_length))))
    ;; Put the left-hand list on the return stack
    (if (result i32)
      ;; If the left-hand list has no items, dispose the partition list in favour of the empty list
      (i32.eqz (local.get $left_length))
      (then
        (call $Term::drop (local.get $self))
        (call $List::empty))
      (else
        ;; Otherwise initialize the left-hand list with its final length
        (call $List::init (local.get $self) (local.get $left_length))))
    ;; Calculate the correct offset for the right-hand list term, ensuring enough space for the list header fields
    (local.set $right_offset
      (call $List::get_item_pointer
        (local.get $self)
        (i32.sub
          (i32.add (local.get $right_offset) (i32.const 1))
          ;; Allow additional space for the list header fields
          (i32.add (global.get $Term::NUM_HEADER_FIELDS) (global.get $List::NUM_HEADER_FIELDS)))))
    ;; Manually set the term header fields to define the right-hand list term
    (call $Term::set_type (local.get $right_offset) (global.get $TermType::List))
    (call $Term::set_num_fields (local.get $right_offset) (i32.add (global.get $List::NUM_HEADER_FIELDS) (local.get $right_length)))
    ;; Initialize the right-hand list with its final length
    (call $List::init (local.get $right_offset) (local.get $right_length)))

  (func $List::init_partition_list_ordered (param $self i32) (result i32 i32)
    (local $right i32)
    ;; Split the list into two partitions at the correct boundary (the right-hand partition will have its order reversed)
    (call $List::init_partition_list_unordered (local.get $self))
    ;; Reverse the right-hand partition to restore the original order
    (call $List::reverse_in_place (local.tee $right))
    (local.get $right)))
