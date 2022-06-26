;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $String::EMPTY (mut i32) (i32.const -1))
  (global $String::NUM_HEADER_FIELDS i32 (i32.const 1))

  (func $String::startup
    ;; Pre-allocate the singleton instance
    (local $instance i32)
    ;; Allocate a new struct of the required size and type (one field for the string length; no additional bytes)
    (local.tee $instance (call $Term::new (global.get $TermType::String) (i32.const 1)))
    ;; Store the string length as the first field
    (call $Term::set_field (local.get $instance) (i32.const 0) (i32.const 0))
    ;; Instantiate the term
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $String::EMPTY))

  (func $String::allocate (export "allocateString") (param $capacity i32) (result i32)
    ;; Allocates a new String term with the given capacity in bytes, allowing data to be copied directly into the allocated array.
    ;; The string must be instantiated before it can be used.
    ;; TODO: Enforce unique interned strings
    (if (result i32)
      (i32.eq (local.get $capacity) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $String::EMPTY))
      (else
        ;; Allocate a new struct of the required size and type (one field for the string length plus one field per 4 bytes)
        (call $Term::new (global.get $TermType::String) (i32.add (global.get $String::NUM_HEADER_FIELDS) (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $capacity)) (i32.const 4)))))))

  (func $String::init (export "initString") (param $self i32) (param $length i32) (result i32)
    ;; This assumes the given string has already been allocated and filled with data
    ;; Store the string length as the first field
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $length))
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $String::allocate_unsized (result i32)
    ;; Allocate a new struct of the required size and type (one field for the string length; no additional bytes)
    (call $Term::new (global.get $TermType::String) (global.get $String::NUM_HEADER_FIELDS)))

  (func $String::init_unsized (param $self i32) (param $length i32) (result i32)
    ;; Update the term size with the correct capacity
    (call $Term::set_num_fields (local.get $self) (i32.add (i32.const 1) (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $length)) (i32.const 4))))
    ;; Instantiate the term
    (call $String::init (local.get $self) (local.get $length)))

  (func $String::from_slice (param $offset i32) (param $length i32) (result i32)
    (local $self i32)
    ;; Allocates a new String term whose contents is copied from the given slice of linear memory
    (if (result i32)
      (i32.eq (local.get $length) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $String::EMPTY))
      (else
        ;; Allocate a new struct of the required size and type (one field for the string length plus one field per 4 bytes)
        (local.tee $self (call $String::allocate (local.get $length)))
        (memory.copy
          (call $String::get::offset (local.get $self))
          (local.get $offset)
          (local.get $length))
        (call $String::init (local.get $length)))))

  (func $String::empty (result i32)
    (global.get $String::EMPTY))

  (func $String::from_char (param $char i32) (result i32)
    (local $instance i32)
    (local.tee $instance (call $String::allocate (i32.const 1)))
    (i32.store8 (call $String::get_char_pointer (local.get $instance) (i32.const 0)) (local.get $char))
    (call $String::init (i32.const 1)))

  (func $String::is (export "isString") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::String) (call $Term::get_type (local.get $term))))

  (func $String::get::length (export "getStringLength") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $String::get::offset (export "getStringOffset") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field_pointer (local.get $self) (i32.const 1)))

  (func $String::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $String::traits::is_atomic (param $self i32) (result i32)
    (call $String::traits::is_static (local.get $self)))

  (func $String::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $String::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; Hash the struct field values
    (call $Hash::write_bytes (local.get $state) (call $String::get::offset (local.get $self)) (call $String::traits::length (local.get $self))))

  (func $String::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (if (result i32)
      (i32.ne (call $String::traits::length (local.get $self)) (call $String::traits::length (local.get $other)))
      (then
        (global.get $FALSE))
      (else
        ;; Deep string comparison
        ;; TODO: Remove deep string comparison in favour of unique interned strings
        (call $Utils::i32_array::equals
          ;; (it's safe to interpret strings as arrays of 4-byte i32 values as the allocator ensures strings are 4-byte-aligned)
          (call $String::get::offset (local.get $self)) (i32.div_u (call $Allocator::pad_to_4_byte_offset (call $String::traits::length (local.get $self))) (i32.const 4))
          (call $String::get::offset (local.get $other)) (i32.div_u (call $Allocator::pad_to_4_byte_offset (call $String::traits::length (local.get $other))) (i32.const 4))))))

  (func $String::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (i32.add
      (local.get $offset)
      (call $Utils::bytes::write_json
        (call $String::get::offset (local.get $self))
        (call $String::get::length (local.get $self))
        (local.get $offset))))

  (func $String::traits::length (param $self i32) (result i32)
    (call $String::get::length (local.get $self)))

  (func $String::traits::collect (param $iterator i32) (param $num_sources i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $List::traits::collect (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (call $String::collect_string_list)
    (local.get $dependencies))

  (func $String::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $List::traits::collect_strict (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (call $String::collect_string_list)
    (local.get $dependencies))

  (func $String::collect_string_list (param $sources i32) (result i32)
    (local $source_count i32)
    (local $source_index i32)
    (local $length i32)
    (local $instance i32)
    (local $target_offset i32)
    (local $source i32)
    (local $source_length i32)
    (local $type_error i32)
    (local.set $source_count (call $List::get::length (local.get $sources)))
    (if (result i32)
      ;; If no source strings were provided, return the empty string
      (i32.eqz (local.get $source_count))
      (then
        (call $String::empty))
      (else
        ;; Otherwise compute the combined length of all the source strings, returning a type error
        ;; if any non-strings were encountered
        (local.set $type_error (global.get $NULL))
        (loop $LOOP
          (if
            ;; If the current item is not a string, update the combined error
            (i32.eqz
              (call $String::is
                (local.tee $source (call $List::get_item (local.get $sources) (local.get $source_index)))))
            (then
              (local.set $type_error
                (call $Signal::traits::union
                  (local.get $type_error)
                  (call $Signal::of
                    (call $Condition::type_error (global.get $TermType::String) (local.get $source))))))
            (else
              ;; Otherwise increase the combined string length
              (local.set $length (i32.add (local.get $length) (call $String::get::length (local.get $source))))))
          ;; If this was not the last source string, continue with the next item
          (br_if $LOOP (i32.lt_u (local.tee $source_index (i32.add (local.get $source_index) (i32.const 1))) (local.get $source_count))))
        (@switch
          (@list
            ;; If a type error was encountered, return the combined type error signal
            (@list
              (i32.ne (global.get $NULL) (local.get $type_error))
              (return (local.get $type_error)))
            ;; If all the source strings were empty, return the empty string
            (@list
              (i32.eqz (local.get $length))
              (return (call $String::empty))))
          ;; Otherwise allocate a string to hold the combined result
          (local.tee $instance (call $String::allocate (local.get $length)))
          ;; Copy the source string contents into the result string
          (local.set $source_index (i32.const 0))
          (local.set $length (i32.const 0))
          (local.set $target_offset (call $String::get::offset (local.get $instance)))
          (loop $LOOP
            (memory.copy
              (i32.add (local.get $target_offset) (local.get $length))
              (call $String::get::offset (local.tee $source (call $List::get_item (local.get $sources) (local.get $source_index))))
              (local.tee $source_length (call $String::get::length (local.get $source))))
            (local.set $length (i32.add (local.get $length) (local.get $source_length)))
            (br_if $LOOP (i32.lt_u (local.tee $source_index (i32.add (local.get $source_index) (i32.const 1))) (local.get $source_count))))
          ;; Initialize the string term
          (call $String::init (local.get $length))))))

  (func $String::get_char_pointer (param $self i32) (param $index i32) (result i32)
    (i32.add
      (call $Term::get_field_pointer (local.get $self) (i32.const 1))
      (local.get $index)))

  (func $String::slice (param $self i32) (param $offset i32) (param $length i32) (result i32)
    (local $instance i32)
    (local $source_length i32)
    (if (result i32)
      ;; If the specified region encompasses the whole string, return the unmodified string
      (i32.and
        (i32.eqz (local.get $offset))
        (i32.ge_u (local.get $length) (local.tee $source_length (call $String::get::length (local.get $self)))))
      (then
        (local.get $self))
      (else
        ;; Otherwise if the specified offset is beyond the end of the string, or if the length is zero, return the empty string
        (if (result i32)
          (i32.or
            (i32.ge_u (local.get $offset) (local.get $source_length))
            (i32.eqz (local.get $length)))
          (then
            (call $String::empty))
          (else
            ;; Otherwise allocate a new string of the correct length
            (local.tee $instance
              (call $String::allocate
                (local.tee $length
                  (call $Utils::i32::min_u
                    (local.get $length)
                    (i32.sub (local.get $source_length) (local.get $offset))))))
            ;; Copy the existing contents into the new string
            (memory.copy
              (call $String::get_char_pointer (local.get $instance) (i32.const 0))
              (call $String::get_char_pointer (local.get $self) (local.get $offset))
              (local.get $length))
            ;; Instantiate the new string
            (call $String::init (local.get $length)))))))

  (func $String::find_index (param $self i32) (param $offset i32) (param $pattern_offset i32) (param $pattern_length i32) (result i32)
    (local $length i32)
    (local $max_offset i32)
    (if (result i32)
      (i32.eqz (local.get $pattern_length))
      (then
        ;; If the pattern is the empty string, return index 0
        (i32.const 0))
      (else
        (if (result i32)
          ;; Otherwise if the pattern is longer than the input string, return the null sentinel value
          (i32.gt_u (local.get $pattern_length) (local.tee $length (call $String::get::length (local.get $self))))
          (then
            (global.get $NULL))
          (else
            ;; Otherwise iterate through the string contents looking for a match
            (local.set $offset (call $String::get_char_pointer (local.get $self) (local.get $offset)))
            (local.set $max_offset (call $String::get_char_pointer (local.get $self) (i32.sub (local.get $length) (local.get $pattern_length))))
            (loop $LOOP
              ;; If the pattern exists at the current index, return the current index
              (if
                (call $Utils::bytes::equals
                  (local.get $offset)
                  (local.get $pattern_length)
                  (local.get $pattern_offset)
                  (local.get $pattern_length))
                (then
                  (return (i32.sub (local.get $offset) (call $String::get::offset (local.get $self)))))
                (else
                  ;; Otherwise continue from the next character until the entire string has been iterated
                  (br_if $LOOP
                    (i32.le_u
                      (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))
                      (local.get $max_offset))))))
            ;; The entire string has been iterated without finding a match; return the null sentinel value
            (global.get $NULL))))))

  (func $String::replace (param $self i32) (param $pattern_offset i32) (param $pattern_length i32) (param $replacement_offset i32) (param $replacement_length i32) (result i32)
    (local $match_index i32)
    (local $instance i32)
    (local $input_length i32)
    (local $output_length i32)
    ;; Find the first index of the given pattern
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $match_index (call $String::find_index (local.get $self) (i32.const 0) (local.get $pattern_offset) (local.get $pattern_length))))
      (then
        ;; If the pattern was not found, return the unmodified string
        (local.get $self))
      (else
        ;; Otherwise create a new string with the pattern replaced
        (local.set $output_length
          (i32.add
            (local.tee $input_length (call $String::get::length (local.get $self)))
            (i32.sub (local.get $replacement_length) (local.get $pattern_length))))
        (local.tee $instance
          (if (result i32)
            (i32.eq (local.get $output_length) (local.get $input_length))
            (then
              ;; If the replacement is the same length as the original pattern, create a clone of the input string
              (call $Term::traits::clone (local.get $self)))
            (else
              ;; Otherwise allocate a new string with the correct length
              (local.tee $instance
                (call $String::allocate (local.get $output_length)))
              ;; Copy the string contents before the pattern match into the output string
              (memory.copy
                (call $String::get::offset (local.get $instance))
                (call $String::get::offset (local.get $self))
                (local.get $match_index))
              ;; Copy the string contents after the pattern match into the output string
              (memory.copy
                (call $String::get_char_pointer
                  (local.get $instance)
                  (i32.add (local.get $match_index) (local.get $replacement_length)))
                (call $String::get_char_pointer
                  (local.get $self)
                  (i32.add (local.get $match_index) (local.get $pattern_length)))
                (i32.sub
                  (local.get $input_length)
                  (i32.add (local.get $match_index) (local.get $pattern_length)))))))
        ;; Copy the replacement string into the new string
        (memory.copy
          (call $String::get_char_pointer (local.get $instance) (local.get $match_index))
          (local.get $replacement_offset)
          (local.get $replacement_length))
        ;; Instantiate the term
        (call $String::init (local.get $output_length)))))

  (func $String::split (param $self i32) (param $pattern_offset i32) (param $pattern_length i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local $previous_offset i32)
    (local $results i32)
    (local $num_matches i32)
    (if (result i32)
      ;; If the source string is empty, no need to test for matches
      (i32.eqz (local.tee $length (call $String::get::length (local.get $self))))
      (then
        (if (result i32)
          (i32.eqz (local.get $pattern_length))
          (then
            ;; If both the source string and the search pattern are empty, return the empty list
            (call $List::empty))
          (else
            ;; Otherwise return a 1-item list containing the empty string
            (call $List::of (call $String::empty)))))
      (else
        ;; Otherwise if the pattern is the empty string, return a list of the string characters
        (if (result i32)
          (i32.eqz (local.get $pattern_length))
          (then
            ;; Allocate a new list of the correct length
            (local.tee $results (call $List::allocate (local.get $length)))
            ;; Iterate through the source string adding each character to the list
            (loop $LOOP
              (call $List::set_item
                (local.get $results)
                (local.get $index)
                (call $String::from_char (i32.load8_u (call $String::get_char_pointer (local.get $self) (local.get $index)))))
              ;; Continue with the next character until the end of the source string is reached
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $length))))
            ;; Instantiate the new list
            (call $List::init (local.get $length)))
          (else
            ;; Otherwise return a list of substrings determined by searching within the string for matches.
            ;; We don't know in advance how many matches there are, so we construct the results list in two stages:
            ;; first store the list of offsets, then once we know the number of matches, overwrite the offsets with the
            ;; actual substring terms. We do this in two stages because we cannot perform any allocations between the
            ;; creation of the unsized list until the last result has been allocated. So first we construct the list
            ;; entries, then once the list has been fully allocated we can begin allocating strings for the contents.
            ;; Note that this is unsafe, seeing as the list is meant to contain term pointers but we are abusing it to
            ;; store raw integers, however the raw integers will be immediately overwritten by the substring terms
            ;; as soon as all the matches have been found.
            ;; Allocate an unsized list in which to store the pattern indices
            (local.set $results (call $List::allocate_unsized))
            ;; Iterate through the source string searching for matches, and store them in the results list
            (loop $LOOP
              (if
                ;; If the pattern was not found, we have reached the end of the string
                (i32.eq
                  (global.get $NULL)
                  (local.tee $index
                    (call $String::find_index
                      (local.get $self)
                      (local.get $index)
                      (local.get $pattern_offset)
                      (local.get $pattern_length))))
                (then)
                (else
                  ;; Otherwise store the match offset in the results list
                  (local.set $results (call $List::grow_unsized (local.get $results) (local.get $index)))
                  ;; If we have not yet reached the end of the source string, search for the next match
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (local.get $index) (local.get $pattern_length)))
                      (local.get $length))))))
            (if (result i32)
              (i32.eqz (local.tee $num_matches (call $List::get::length (local.get $results))))
              (then
                ;; If there were no matches, return a list containing the whole unmodified source string
                (call $List::of (local.get $self)))
              (else
                ;; Append a final substring that spans to the end of the string
                (local.set $results (call $List::grow_unsized (local.get $results) (local.get $length)))
                ;; Now that we have found all the matches, iterate through the results
                ;; creating substrings for each of the matches and overwring the contents of the results list
                (local.set $index (i32.const 0))
                (loop $LOOP
                  ;; Overwrite the current list item
                  (call $List::set_item
                    (local.get $results)
                    (local.get $index)
                    ;; Calculate offset and length values for the current substring
                    (call $String::create_slice_from_start_end_offsets
                      ;; Start the substring at the end of the previous match, or zero if this is the first iteration
                      (call $String::get_char_pointer
                        (local.get $self)
                        (select
                          (i32.const 0)
                          (i32.add (local.get $previous_offset) (local.get $pattern_length))
                          (i32.eqz (local.get $index))))
                      ;; End the substring at the address of the next match (retrieved from the results list)
                      (call $String::get_char_pointer
                        (local.get $self)
                        (local.tee $previous_offset
                          ;; Load the integer value of the result offset from the results list
                          (i32.load (call $List::get_item_pointer (local.get $results) (local.get $index))))))
                    ;; Allocate a substring with the given offset and length
                    (call $String::from_slice))
                  ;; Advance to the next match
                  (br_if $LOOP
                    (i32.le_u
                      (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                      (local.get $num_matches))))
                ;; Instantiate the list of substrings
                (local.get $results)
                (call $List::init_unsized))))))))

  (func $String::create_slice_from_start_end_offsets (param $start i32) (param $end i32) (result i32 i32)
    (local.get $start)
    (i32.sub (local.get $end) (local.get $start))))
