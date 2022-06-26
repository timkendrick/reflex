;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $List
    (@struct $List
      (@field $items (@repeated (@ref $Term))))

    (@derive $size (@get $List))
    (@derive $hash (@get $List))

    (@export $List (@get $List)))

  (export "isList" (func $Term::List::is))
  (export "getListLength" (func $Term::List::get::items::length))

  (@const $Term::List::EMPTY i32 (call $Term::TermType::List::new))

  ;; Minimum list capacity when allocating non-zero-length lists of unknown size
  (global $Term::List::MIN_UNSIZED_LIST_CAPACITY i32 (i32.const 8))

  (func $List::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; This assumes that lists with the same length and hash are almost certainly identical
    (i32.eq
      (call $List::get::items::length (local.get $self))
      (call $List::get::items::length (local.get $other))))

  (func $Term::List::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the list items pointer for an imaginary list term located at
    ;; memory address 0. The pointer offset tells us how many bytes are taken up by the preceding list wrapper.
    (call $Term::List::get::items::pointer (i32.const 0) (i32.const 0)))

  (func $Term::List::allocate (export "allocateList") (param $capacity i32) (result i32)
    ;; Allocates a new List term with the given capacity, allowing items to be copied directly into the allocated slots.
    ;; The list must be instantiated before it can be used.
    ;; TODO: Investigate unique interned lists
    (local $self i32)
    (if (result i32)
      (i32.eqz (local.get $capacity))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::List::EMPTY))
      (else
        ;; The standard constructor wrappers take care of allocating space for a standard term,
        ;; however they do not allocate space for extra elements as needed by the list term.
        ;; This means we have to manually allocate a larger amount of space than usual,
        ;; then fill in the list term contents into the newly-allocated space.
        ;; First allocate a new term wrapper with the required capacity
        (local.tee $self
          (call $Allocator::allocate
            (i32.add
              (call $Term::List::empty::sizeof)
              (i32.mul (i32.const 4) (local.get $capacity)))))
        ;; Then manually write the list struct contents into the term wrapper
        (call $TermType::List::construct (call $Term::pointer::value (local.get $self)))
        (call $Term::List::set::items::capacity (local.get $self) (local.get $capacity)))))

  (func $Term::List::init (export "initList") (param $self i32) (param $length i32) (result i32)
    ;; This assumes the given list has already been allocated and filled with items
    ;; Store the list length
    (call $Term::List::set::items::length (local.get $self) (local.get $length))
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $Term::List::allocate_unsized (result i32)
    ;; This creates a new list of unknown size.
    ;; The list MUST be filled and initialized before any further allocations can take place.
    (call $Term::List::empty))

  (func $Term::List::grow_unsized (param $self i32) (param $value i32) (result i32)
    ;; This extends an existing unsized list into unallocated free space.
    ;; This will panic if the list is not the most recent heap object to have been allocated.
    ;; This means that this operation is only suitable for synchronous 'unsafe' list creation,
    ;; where it is certain that no other heap objects will be allocated during the list construction.
    (local $capacity i32)
    (local $length i32)
    (if (result i32)
      ;; If this is the first item (as indicated by the list being empty), allocate a new non-empty list
      (i32.eqz (local.tee $capacity (call $Term::List::get::items::capacity (local.get $self))))
      (then
        ;; Allocate a new list struct with capacity for a single item
        (local.tee $self (call $Term::List::allocate (i32.const 1)))
        ;; Insert the value as the first list item
        (call $Term::List::set::items::length (local.get $self) (i32.const 1))
        (call $Term::List::set::items::value (local.get $self) (i32.const 0) (local.get $value)))
      (else
        ;; Otherwise determine whether the list has space for another list item, extending the existing allocation if necessary
        (if
          (i32.eq
            (local.tee $length (call $Term::List::get::items::length (local.get $self)))
            (local.get $capacity))
          (then
            ;; Allocate space for a single list item, and assert that the allocated slot is contiguous with the existing items
            (if
              (i32.ne
                (call $Allocator::allocate (i32.mul (i32.const 4) (i32.const 1)))
                (call $Term::List::get::items::pointer
                  (local.get $self)
                  (local.get $capacity)))
              (then
                ;; Panic if the allocated slot is not contiguous. This will happen if there were intermediate allocations
                ;; unrelated to the list building since the most recent item was added
                (unreachable))
              (else
                ;; Update the list capacity
                (call $Term::List::set::items::capacity (local.get $self) (i32.add (local.get $capacity) (i32.const 1))))))
          (else))
        ;; Push the item onto the list and update the list length
        (call $Term::List::set::items::length (local.get $self) (i32.add (local.get $length) (i32.const 1)))
        (call $Term::List::set::items::value (local.get $self) (local.get $length) (local.get $value))
        (local.get $self))))

  (func $Term::List::append_unsized (param $self i32) (param $value i32) (result i32)
    ;; This extends an existing unsized list into unallocated free space if it is the most recent heap object to have been allocated,
    ;; or reallocates a larger list when necessary if there have been more recent heap allocations
    (local $capacity i32)
    (local $length i32)
    (if (result i32)
      ;; If this is the first list item, allocate a new list
      (i32.eqz (local.tee $capacity (call $Term::List::get::items::capacity (local.get $self))))
      (then
        ;; Allocate a new list struct with the minimum dynamic capacity
        (local.tee $self (call $Term::List::allocate (global.get $Term::List::MIN_UNSIZED_LIST_CAPACITY)))
        (call $Term::List::set::items::length (local.get $self) (i32.const 1))
        (call $Term::List::set::items::value (local.get $self) (i32.const 0) (local.get $value)))
      (else
        ;; Otherwise determine whether the list has space for another list item, reallocating to a larger slot if necessary
        (if
          (i32.eq
            (local.tee $length (call $Term::List::get::items::length (local.get $self)))
            (local.get $capacity))
          (then
            ;; Reallocate the list with double the capacity
            (local.set $self (call $Term::List::reallocate (local.get $self) (i32.mul (local.get $length) (i32.const 2)))))
          (else))
        ;; Push the item onto the list and update the list length
        (call $Term::List::set::items::length (local.get $self) (i32.add (local.get $length) (i32.const 1)))
        (call $Term::List::set::items::value (local.get $self) (local.get $length) (local.get $value))
        (local.get $self))))

  (func $Term::List::init_unsized (param $self i32) (result i32)
    (local $length i32)
    (if (result i32)
      ;; If there were no items added to the list, return the empty list
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (call $Term::List::empty))
      (else
        ;; Otherwise instantiate the term
        (call $Term::List::init (local.get $self) (local.get $length)))))

  (func $Term::List::empty (export "createEmptyList") (result i32)
    ;; Allocate a new list of the required capacity
    ;; (this will return the pre-allocated empty list singleton)
    (call $Term::List::allocate (i32.const 0)))

  (func $Term::List::of (export "createUnitList") (param $value i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required capacity
    (local.tee $self (call $Term::List::allocate (i32.const 1)))
    ;; Store the list items at the correct offsets
    (call $Term::List::set::items::value (local.get $self) (i32.const 0) (local.get $value))
    ;; Instantiate the list term with the correct length
    (call $Term::List::init (i32.const 1)))

  (func $Term::List::create_pair (export "createPair") (param $left i32) (param $right i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required capacity
    (local.tee $self (call $Term::List::allocate (i32.const 2)))
    ;; Store the list items at the correct offsets
    (call $Term::List::set::items::value (local.get $self) (i32.const 0) (local.get $left))
    (call $Term::List::set::items::value (local.get $self) (i32.const 1) (local.get $right))
    ;; Instantiate the list term with the correct length
    (call $Term::List::init (i32.const 2)))

  (func $Term::List::create_triple (export "createTriple") (param $first i32) (param $second i32) (param $third i32) (result i32)
    (local $self i32)
    ;; Allocate a new list of the required capacity
    (local.tee $self (call $Term::List::allocate (i32.const 3)))
    ;; Store the list items at the correct offsets
    (call $Term::List::set::items::value (local.get $self) (i32.const 0) (local.get $first))
    (call $Term::List::set::items::value (local.get $self) (i32.const 1) (local.get $second))
    (call $Term::List::set::items::value (local.get $self) (i32.const 2) (local.get $third))
    ;; Instantiate the list term with the correct length
    (call $Term::List::init (i32.const 3)))

  (func $Term::List::traits::is_atomic (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return true
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is not atomic, return false
            (i32.eqz (call $Term::traits::is_atomic (call $Term::List::get::items::value (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no non-atomic items were encountered in the entire list, return true
        (global.get $TRUE))))

  (func $Term::List::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::List::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local $item i32)
    (local $substituted_item i32)
    (local $results i32)
    (if (result i32)
      ;; If the list is empty, return the unmodified marker
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (global.get $NULL))
      (else
        ;; Otherwise iterate through each list item in turn
        (local.set $results (global.get $NULL))
        (loop $LOOP
          ;; Get the substituted item value
          (local.set $substituted_item
            (call $Term::traits::substitute
              (local.tee $item (call $Term::List::get::items::value (local.get $self) (local.get $index)))
              (local.get $variables)
              (local.get $scope_offset)))
          (if
            ;; If the item was modified, and this is the first item to have been modified, create a new results list
            (i32.and
              (i32.ne (global.get $NULL) (local.get $substituted_item))
              (i32.eq (global.get $NULL) (local.get $results)))
            (then
              ;; Create a new result list term with the correct size
              (local.set $results (call $Term::List::allocate (local.get $length)))
              ;; Copy any previous items into the results list
              (if
                (i32.eqz (local.get $index))
                (then)
                (else
                  (memory.copy
                    (call $Term::List::get::items::pointer (local.get $results) (i32.const 0))
                    (call $Term::List::get::items::pointer (local.get $self) (i32.const 0))
                    (i32.mul (i32.const 4) (local.get $index)))))
              ;; Push the substituted item onto the results list
              (call $Term::List::set::items::value
                (local.get $results)
                (local.get $index)
                (local.get $substituted_item)))
            (else
              ;; Otherwise if there have been modifications to the preceding items,
              ;; Push the current result onto the results list
              (if
                (i32.ne (global.get $NULL) (local.get $results))
                (then
                  (call $Term::List::set::items::value
                    (local.get $results)
                    (local.get $index)
                    ;; Add the unmodified value or the substituted value as appropriate
                    (select
                      (local.get $item)
                      (local.get $substituted_item)
                      (i32.eq (global.get $NULL) (local.get $substituted_item)))))
                ;; Otherwise nothing more needs to be done for this item
                (else))))
          ;; Continue with the next item
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))
        ;; If there were any substitutions, return the initialized results list term
        (if (result i32)
          (i32.ne (global.get $NULL) (local.get $results))
          (then
            (call $Term::List::init (local.get $results) (local.get $length)))
          (else
            ;; Otherwise return the unmodified marker
            (global.get $NULL))))))

  (func $Term::List::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $length i32)
    (local $index i32)
    (local $item i32)
    (if (result i32 i32)
      ;; If the list is empty, write an empty JSON array literal
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        ;; Put the success marker on the stack
        (global.get $TRUE)
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
          ;; If the value does not support JSON serialization, bail out
          (if
            (i32.eqz
              (call $Term::implements::to_json
                (local.tee $item (call $Term::List::get::items::value (local.get $self) (local.get $index)))))
            (then
              (return (global.get $FALSE) (local.get $offset)))
            (else))
          ;; Write the current item to the output and store the updated offset
          (local.set $offset
            (call $Term::traits::to_json
              (local.get $item)
              ;; The target offset is incremented to reflect the preceding 1-byte opening brace or character separator
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
              (@char "]")
              (@char ",")
              (i32.eq (local.get $index) (i32.sub (local.get $length) (i32.const 1)))))
          ;; If this was not the final item, continue with the next item
          (br_if $LOOP
            (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Return the updated offset, taking into account the final closing brace
        (i32.add (local.get $offset) (i32.const 1)))))

  (func $Term::List::traits::length (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::List::get::items::length (local.get $self)))

  (func $Term::List::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::List::traits::size_hint (param $self i32) (result i32)
    (call $Term::List::traits::length (local.get $self)))

  (func $Term::List::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      ;; If we have iterated through all the items, return the complete marker
      (i32.eq
        ;; Get the current iterator index from the state (initializing to zero if this is the first iteration)
        (local.tee $iterator_state
          (select
            (i32.const 0)
            (local.get $iterator_state)
            (i32.eq (global.get $NULL) (local.get $iterator_state))))
        (call $Term::List::get::items::length (local.get $self)))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Otherwise emit the current item and the incremented iterator state
        (call $Term::List::get::items::value (local.get $self) (local.get $iterator_state))
        (i32.add (local.get $iterator_state) (i32.const 1))
        (global.get $NULL))))

  (func $Term::List::traits::get (param $self i32) (param $key i32) (result i32)
    (local $index i32)
    (if (result i32)
      ;; If the key is not an integer, return the null sentinel value
      (i32.eqz (call $Term::Int::is (local.get $key)))
      (then
        (global.get $NULL))
      (else
        (if (result i32)
          ;; Determine whether the index is within the list bounds
          (i32.and
            (i32.ge_s (local.tee $index (call $Term::Int::get::value (local.get $key))) (i32.const 0))
            (i32.lt_u (local.get $index) (call $Term::List::get::items::length (local.get $self))))
          (then
            ;; If the index is within the list bounds, etrieve the corresponding list item
            (call $Term::List::get::items::value (local.get $self) (local.get $index)))
          (else
            ;; Otherwise return the null sentinel value
            (global.get $NULL))))))

  (func $Term::List::traits::has (param $self i32) (param $key i32) (result i32)
    (local $index i32)
    (if (result i32)
      ;; If the key is not an integer, return false
      (i32.eqz (call $Term::Int::is (local.get $key)))
      (then
        (global.get $FALSE))
      (else
        ;; Determine whether the index is within the list bounds
        (i32.and
          (i32.ge_s (local.tee $index (call $Term::Int::get::value (local.get $key))) (i32.const 0))
          (i32.lt_u (local.get $index) (call $Term::List::get::items::length (local.get $self)))))))

  (func $Term::List::traits::keys (param $self i32) (result i32)
    (call $Term::RangeIterator::new (i32.const 0) (call $Term::List::get::items::length (local.get $self))))

  (func $Term::List::traits::values (param $self i32) (result i32)
    (call $Term::List::traits::iterate (local.get $self)))

  (func $Term::List::traits::union (param $self i32) (param $other i32) (result i32)
    (local $result i32)
    (local $length_self i32)
    (local $length_other i32)
    (local.set $length_self (call $Term::List::get::items::length (local.get $self)))
    (local.set $length_other (call $Term::List::get::items::length (local.get $other)))
    (local.tee $result (call $Term::List::allocate (i32.add (local.get $length_self) (local.get $length_other))))
    (memory.copy
      (call $Term::List::get::items::pointer (local.get $result) (i32.const 0))
      (call $Term::List::get::items::pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $length_self) (i32.const 4)))
    (memory.copy
      (call $Term::List::get::items::pointer (local.get $result) (local.get $length_self))
      (call $Term::List::get::items::pointer (local.get $other) (i32.const 0))
      (i32.mul (local.get $length_other) (i32.const 4)))
    ;; Instantiate the list term
    (call $Term::List::init (i32.add (local.get $length_self) (local.get $length_other))))

  (func $Term::List::get_length (param $self i32) (result i32)
    (call $Term::List::get::items::length (local.get $self)))

  (func $Term::List::get_items (export "getListItems") (param $self i32) (result i32)
    (call $Term::List::get::items::pointer (local.get $self) (i32.const 0)))

  (func $Term::List::get_item (export "getListItem") (param $self i32) (param $index i32) (result i32)
    (call $Term::List::get::items::value (local.get $self) (local.get $index)))

  (func $Term::List::set_item (param $self i32) (param $index i32) (param $value i32)
    (call $Term::List::set::items::value (local.get $self) (local.get $index) (local.get $value)))

  (func $Term::List::update_index (param $self i32) (param $index i32) (param $value i32) (result i32)
    ;; Return a clone of the given list, with the specified index set to the given value
    ;; This assumes the given index is within the list bounds
    ;; Create a clone of the original list
    (local.tee $self (call $Term::traits::clone (local.get $self)))
    ;; Update the specified index with the specified value
    (call $Term::List::set::items::value (local.get $self) (local.get $index) (local.get $value))
    ;; Instantiate the term
    (call $Term::init))

  (func $Term::List::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a list, return the existing instance
      (call $Term::List::is (local.get $iterator))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the list items according to whether the iterator size is known
        (if (result i32 i32)
          (i32.eq (local.tee $length (call $Term::traits::size_hint (local.get $iterator))) (global.get $NULL))
          (then
            (call $Term::List::collect_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Term::List::collect_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Term::List::collect_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty list
      (i32.eqz (local.get $length))
      (then
        (call $Term::List::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new list to hold the results and fill it by consuming each iterator item in turn
        (local.tee $instance (call $Term::List::allocate (local.get $length)))
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
              (call $Term::List::set::items::value (local.get $instance) (local.get $index) (local.get $item))
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $LOOP))))
        ;; Initialize the list term
        (call $Term::List::init (local.get $index))
        (local.get $dependencies))))

  (func $Term::List::collect_unsized (param $iterator i32) (param $state i32) (result i32 i32)
    ;; We cannot know in advance the correct size of list to allocate, so we start off with the empty list, then
    ;; allocate a series of lists of doubling capacity as more iterator items are consumed from the source iterator
    (local $instance i32)
    (local $capacity i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    ;; Start off with the empty list to avoid an unnecessary allocation for empty source iterators
    (local.set $instance (call $Term::List::empty))
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
              (call $Term::List::set::items::length (local.get $instance) (local.get $index))
              ;; Reallocate the list to a new location with double the capacity
              (local.set $instance
                (call $Term::List::reallocate
                  (local.get $instance)
                  (local.tee $capacity
                    (select
                      ;; If this is the first non-empty allocation, create a list of a predetermined capacity
                      (global.get $Term::List::MIN_UNSIZED_LIST_CAPACITY)
                      ;; Otherwise create a new list with double the existing capacity
                      ;; (this ensures amortized list allocations as the number of items increases)
                      (i32.mul (local.get $capacity) (i32.const 2))
                      (i32.eqz (local.get $capacity)))))))
            (else))
          ;; Store the item in the results list and continue with the next item
          (call $Term::List::set::items::value (local.get $instance) (local.get $index) (local.get $item))
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
        (call $Term::List::init (local.get $instance) (local.get $index))
        (local.get $dependencies))))

  (func $Term::List::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $length i32)
    (if (result i32 i32)
      ;; If the source iterator is already a list composed solely of static items, return the existing instance
      (if (result i32)
        (call $Term::List::is (local.get $iterator))
        (then
          (i32.eqz (call $Term::List::has_dynamic_items (local.get $iterator))))
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
            (call $Term::List::collect_strict_unsized (local.get $iterator) (local.get $state)))
          (else
            (call $Term::List::collect_strict_sized (local.get $length) (local.get $iterator) (local.get $state)))))))

  (func $Term::List::collect_strict_sized (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
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
        (call $Term::List::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new list to hold the results and fill it by consuming each iterator item in turn
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $signal (global.get $NULL))
        (local.set $instance (call $Term::List::allocate (local.get $length)))
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
                    (call $Term::Signal::traits::union
                      (local.get $signal)
                      (select
                        (local.get $item)
                        (global.get $NULL)
                        (call $Term::Signal::is (local.get $item))))))
                (then
                  ;; Continue with the next item
                  (local.set $index (i32.add (local.get $index) (i32.const 1)))
                  (br $LOOP))
                (else
                  ;; Otherwise store the item in the results list
                  (call $Term::List::set::items::value (local.get $instance) (local.get $index) (local.get $item))
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
            (call $Term::List::init (local.get $instance) (local.get $index))
            (local.get $dependencies))))))

  (func $Term::List::collect_strict_unsized (param $iterator i32) (param $state i32) (result i32 i32)
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
    (local.set $instance (call $Term::List::empty))
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
                (call $Term::Signal::traits::union
                  (local.get $signal)
                  (select
                    (local.get $item)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $item))))))
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
                  (call $Term::List::set::items::length (local.get $instance) (local.get $index))
                  ;; Reallocate the list to a new location with double the capacity
                  (local.set $instance
                    (call $Term::List::reallocate
                      (local.get $instance)
                      (local.tee $capacity
                        (select
                          ;; If this is the first non-empty allocation, create a list of a predetermined capacity
                          (global.get $Term::List::MIN_UNSIZED_LIST_CAPACITY)
                          ;; Otherwise create a new list with double the existing capacity
                          ;; (this ensures amortized list allocations as the number of items increases)
                          (i32.mul (local.get $capacity) (i32.const 2))
                          (i32.eqz (local.get $capacity)))))))
                (else))
              (call $Term::List::set::items::value (local.get $instance) (local.get $index) (local.get $item))
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
            (call $Term::List::init (local.get $instance) (local.get $index))
            (local.get $dependencies))))))

  (func $Term::List::reallocate (param $self i32) (param $capacity i32) (result i32)
    (local $instance i32)
    (if (result i32)
      ;; If the list already has sufficient capacity, return it as-is
      (i32.ge_u (call $Term::List::get::items::capacity (local.get $self)) (local.get $capacity))
      (then
        (local.get $self))
      (else
        ;; Otherwise allocate a new list with the given capacity
        (local.tee $instance (call $Term::List::allocate (local.get $capacity)))
        ;; If the source list contains any items, copy them across to the new list
        (if
          (i32.eqz (call $Term::List::get::items::length (local.get $self)))
          (then)
          (else
            (memory.copy
              (call $Term::List::get::items::pointer (local.get $instance) (i32.const 0))
              (call $Term::List::get::items::pointer (local.get $self) (i32.const 0))
              (i32.mul (call $Term::List::get::items::length (local.get $self)) (i32.const 4)))))
        ;; Rewrite the source list as a redirect pointer term
        ;; (this is to avoid breaking any existing pointers to the original list address)
        (call $Term::redirect (local.get $self) (local.get $instance)))))

  (func $Term::List::has_dynamic_items (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return false
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (global.get $FALSE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is dynamic, return true
            (i32.eqz (call $Term::is_static (call $Term::List::get::items::value (local.get $self) (local.get $index))))
            (then
              (return (global.get $TRUE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no dynamic items were encountered in the entire list, return false
        (global.get $FALSE))))

  (func $Term::List::find_index (param $self i32) (param $value i32) (result i32)
    (local $num_items i32)
    (local $item_index i32)
    (local.set $num_items (call $Term::List::get::items::length (local.get $self)))
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
            (call $Term::traits::equals (call $Term::List::get::items::value (local.get $self) (local.get $item_index)) (local.get $value))
            (then
              ;; Item matches; return current index
              (local.get $item_index))
            (else
              ;; Try the next item
              (local.set $item_index (i32.add (local.get $item_index) (i32.const 1)))
              (br $LOOP)))))))

  (func $Term::List::push (param $self i32) (param $value i32) (result i32)
    (local $instance i32)
    (local $existing_length i32)
    ;; Allocate a new list with the correct capacity
    (local.tee $instance (call $Term::List::allocate (i32.add (local.tee $existing_length (call $Term::List::get::items::length (local.get $self))) (i32.const 1))))
    ;; Copy the existing values into the new list
    (memory.copy
      (call $Term::List::get::items::pointer (local.get $instance) (i32.const 0))
      (call $Term::List::get::items::pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $existing_length) (i32.const 4)))
    ;; Add the provided value to the new list
    (call $Term::List::set::items::value (local.get $instance) (local.get $existing_length) (local.get $value))
    ;; Instantiate the new list
    (call $Term::List::init (i32.add (local.get $existing_length) (i32.const 1))))

  (func $Term::List::push_front (param $self i32) (param $value i32) (result i32)
    (local $instance i32)
    (local $existing_length i32)
    ;; Allocate a new list with the correct capacity
    (local.tee $instance (call $Term::List::allocate (i32.add (local.tee $existing_length (call $Term::List::get::items::length (local.get $self))) (i32.const 1))))
    ;; Copy the existing values into the new list
    (memory.copy
      (call $Term::List::get::items::pointer (local.get $instance) (i32.const 1))
      (call $Term::List::get::items::pointer (local.get $self) (i32.const 0))
      (i32.mul (local.get $existing_length) (i32.const 4)))
    ;; Add the provided value to the new list
    (call $Term::List::set::items::value (local.get $instance) (i32.const 0) (local.get $value))
    ;; Instantiate the new list
    (call $Term::List::init (i32.add (local.get $existing_length) (i32.const 1))))

  (func $Term::List::slice (param $self i32) (param $offset i32) (param $length i32) (result i32)
    (local $instance i32)
    (local $source_length i32)
    (if (result i32)
      ;; If the specified region encompasses the whole list, return the unmodified list
      (i32.and
        (i32.eqz (local.get $offset))
        (i32.ge_u (local.get $length) (local.tee $source_length (call $Term::List::get::items::length (local.get $self)))))
      (then
        (local.get $self))
      (else
        ;; Otherwise if the specified offset is beyond the end of the list, or the length is zero, return the empty list
        (if (result i32)
          (i32.or
            (i32.ge_u (local.get $offset) (local.get $source_length))
            (i32.eqz (local.get $length)))
          (then
            (call $Term::List::empty))
          (else
            ;; Otherwise allocate a new list of the correct capacity
            (local.tee $instance
              (call $Term::List::allocate
                (local.tee $length
                  (call $Utils::i32::min_u
                    (local.get $length)
                    (i32.sub (local.get $source_length) (local.get $offset))))))
            ;; Copy the existing items into the new list
            (memory.copy
              (call $Term::List::get::items::pointer (local.get $instance) (i32.const 0))
              (call $Term::List::get::items::pointer (local.get $self) (local.get $offset))
              (i32.mul (local.get $length) (i32.const 4)))
            ;; Instantiate the new list
            (call $Term::List::init (local.get $length)))))))

  (func $Term::List::is_typed_list (param $self i32) (param $type i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return true
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is not of the given type, return false
            (i32.ne (local.get $type) (call $Term::get_type (call $Term::List::get::items::value (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no items not of the given type were encountered in the entire list, return true
        (global.get $TRUE))))

  (func $Term::List::reverse_in_place (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local $right_index i32)
    (local $left_value i32)
    (local $right_value i32)
    (if (result i32)
      ;; If the list is empty or only contains a single item, nothing to do
      (i32.lt_u (local.tee $length (call $Term::List::get::items::length (local.get $self))) (i32.const 2))
      (then
        (local.get $self))
      (else
        ;; Otherwise iterate through each item in the left hand half of the list,
        ;; swapping it with the corresponding item in the right hand half of the list
        (loop $LOOP (result i32)
          ;; Determine the index of the corresponding item in the right-hand half of the list
          (local.set $right_index (i32.sub (i32.sub (local.get $length) (i32.const 1)) (local.get $index)))
          ;; Swap the left-hand and right-hand items
          (local.set $left_value (call $Term::List::get::items::value (local.get $self) (local.get $index)))
          (local.set $right_value (call $Term::List::get::items::value (local.get $self) (local.get $right_index)))
          (call $Term::List::set::items::value (local.get $self) (local.get $index) (local.get $right_value))
          (call $Term::List::set::items::value (local.get $self) (local.get $right_index) (local.get $left_value))
          ;; If we have not yet reached the middle of the list, continue with the next item
          (br_if $LOOP
            (i32.lt_u
              (local.tee $index (i32.add (local.get $index) (i32.const 1)))
              (i32.div_u (local.get $length) (i32.const 2))))
          ;; Update the list hash
          (call $Term::List::init (local.get $self) (local.get $length))))))

  (func $Term::List::allocate_partition_list (param $length i32) (result i32)
    ;; In order to minimize allocations, we allocate a single list that is large enough to store the items from both
    ;; partitions, and then once all items have been added we subdivide the list into two separate sub-lists.
    ;; Seeing as we cannot know in advance how large each partition needs to be, items will be added at either end
    ;; depending on which partition they belong to. The list will be split into two sub-lists when it is initialized.
    (local $instance i32)
    (local $capacity i32)
    (local.tee $instance
      (call $Term::List::allocate
        (local.tee $capacity
          (i32.add
            ;; Seeing as we are creating a single contiguous list that will later be split into two lists, we need to
            ;; allocate additional space for the right-hand partition's list header (to be added retrospectively).
            (i32.div_u (call $Term::List::empty::sizeof) (i32.const 4))
            (local.get $length)))))
    ;; While building the lists, the list length field will be used to track the current offsets of the two partitions.
    ;; Each offset value indicates the field index of the next available slot for the given partition.
    ;; The left partition is filled from the left-hand side; the right partition is filled from the right-hand side.
    ;; This means that the left-hand offset counts up from zero, while the right-hand offset counts down from (len - 1).
    ;; Repurposing the list length field requires that we fit two offsets into a single field; this is done by splitting
    ;; the 32-bit list length field into two 16-bit integers representing the left and right partition offsets.
    ;; Note that this means that neither partition can exceed 2^16 items in length.
    ;; This field will be overridden by the final length of the left-hand partition when the list is initialized.
    (call $Term::List::set_partition_list_offsets
      (local.get $instance)
      (i32.const 0)
      (i32.sub (local.get $capacity) (i32.const 1))))

  (func $Term::List::get_partition_list_offsets (param $self i32) (result i32 i32)
    (local $combined i32)
    (i32.shr_u
      (i32.and (i32.const 0xFFFF0000) (local.tee $combined (call $Term::List::get::items::length (local.get $self))))
      (i32.const 16))
    (i32.and (i32.const 0x0000FFFF) (local.get $combined)))

  (func $Term::List::get_partition_list_offset_left (param $self i32) (result i32)
    (i32.shr_u
      (i32.and (i32.const 0xFFFF0000) (call $Term::List::get::items::length (local.get $self)))
      (i32.const 16)))

  (func $Term::List::get_partition_list_offset_right (param $self i32) (result i32)
    (i32.and (i32.const 0x0000FFFF) (call $Term::List::get::items::length (local.get $self))))

  (func $Term::List::set_partition_list_offsets (param $self i32) (param $left_offset i32) (param $right_offset i32)
    (call $Term::List::set::items::length
      (local.get $self)
      (i32.or
        (i32.shl (i32.and (i32.const 0x0000FFFF) (local.get $left_offset)) (i32.const 16))
        (i32.and (i32.const 0x0000FFFF) (local.get $right_offset)))))

  (func $Term::List::set_partition_list_offset_left (param $self i32) (param $value i32)
    (call $Term::List::set_partition_list_offsets
      (local.get $self)
      (local.get $value)
      (call $Term::List::get_partition_list_offset_right (local.get $self))))

  (func $Term::List::set_partition_list_offset_right (param $self i32) (param $value i32)
    (call $Term::List::set_partition_list_offsets
      (local.get $self)
      (call $Term::List::get_partition_list_offset_left (local.get $self))
      (local.get $value)))

  (func $Term::List::insert_partition_list_item (param $self i32) (param $partition i32) (param $value i32)
    (if
      (i32.eqz (local.get $partition))
      (then
        (call $Term::List::insert_partition_list_item_left (local.get $self) (local.get $value)))
      (else
        (call $Term::List::insert_partition_list_item_right (local.get $self) (local.get $value)))))

  (func $Term::List::insert_partition_list_item_left (param $self i32) (param $value i32)
    (local $offset i32)
    ;; Insert the list item at the current left-hand partition offset
    (i32.store
      (call $Term::List::get::items::pointer
        (local.get $self)
        (local.tee $offset (call $Term::List::get_partition_list_offset_left (local.get $self))))
      (local.get $value))
    ;; Increment the left-hand partition offset field
    (call $Term::List::set_partition_list_offset_left (local.get $self) (i32.add (local.get $offset) (i32.const 1))))

  (func $Term::List::insert_partition_list_item_right (param $self i32) (param $value i32)
    (local $offset i32)
    ;; Insert the list item at the current right-hand partition offset
    (i32.store
      (call $Term::List::get::items::pointer
        (local.get $self)
        (local.tee $offset (call $Term::List::get_partition_list_offset_right (local.get $self))))
      (local.get $value))
    ;; Decrement the right-hand partition offset field
    (call $Term::List::set_partition_list_offset_right (local.get $self) (i32.sub (local.get $offset) (i32.const 1))))

  (func $Term::List::init_partition_list_unordered (param $self i32) (result i32 i32)
    ;; This splits the list at the correct partition boundary, leaving the items in the positions they were added
    ;; (i.e. the left-hand partition will retain the original order, but the right-hand partition will be reversed)
    (local $combined_capacity i32)
    (local $right_offset i32)
    (local $left_length i32)
    (local $right_length i32)
    ;; Determine the length of each partition based on the stored partition offsets
    (call $Term::List::get_partition_list_offsets (local.get $self))
    ;; The right offset was used to indicate the next free slot working backwards from the right-hand side, so to get
    ;; the offset of the last-allocated item we add 1 to the right-hand offset
    (local.set $right_offset (i32.add (i32.const 1)))
    ;; The left offset is identical to the length
    (local.set $left_length)
    (local.set $right_length
      (i32.sub
        (call $Term::List::get::items::capacity (local.get $self))
        (local.get $right_offset)))
    ;; Calculate the correct offset for the right-hand list term wrapper, ensuring enough space for the header fields
    (local.set $right_offset
      (i32.sub
        ;; Get a pointer to the last-inserted right-hand item
        (call $Term::List::get::items::pointer (local.get $self) (local.get $right_offset))
        ;; Subtract additional space to accommodate the list header fields
        (call $Term::List::empty::sizeof)))
    ;; Shrink the left-hand list capacity to the correct size to make space for the right-hand list
    (call $Term::List::set::items::capacity (local.get $self) (local.get $left_length))
    ;; Put the left-hand list on the return stack
    (if (result i32)
      ;; If the left-hand list has no items, dispose the partition list in favour of the empty list
      (i32.eqz (local.get $left_length))
      (then
        (call $Term::drop (local.get $self))
        (call $Term::List::empty))
      (else
        ;; Otherwise initialize the left-hand list with its final length
        (call $Term::List::init (local.get $self) (local.get $left_length))))
    ;; Manually write the list struct term wrapper at the correct offset to define the right-hand list term
    (call $TermType::List::construct (call $Term::pointer::value (local.get $right_offset)))
    (call $Term::List::set::items::capacity (local.get $right_offset) (local.get $right_length))
    ;; Initialize the right-hand list with its final length
    (call $Term::List::init (local.get $right_offset) (local.get $right_length)))

  (func $Term::List::init_partition_list_ordered (param $self i32) (result i32 i32)
    ;; Split the list into two partitions at the correct boundary (the right-hand partition will have its order reversed)
    (call $Term::List::init_partition_list_unordered (local.get $self))
    ;; Reverse the right-hand partition to restore the original order
    (call $Term::List::reverse_in_place)))
