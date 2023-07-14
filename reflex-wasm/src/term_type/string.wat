;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $String
    (@struct $String
      (@field $length i32)
      (@field $data (@repeated i32)))

    (@derive $size (@get $String))
    (@derive $equals (@get $String))
    (@derive $hash (@get $String))

    (@export $String (@get $String)))

  (export "isString" (func $Term::String::is))
  (export "getStringLength" (func $Term::String::get::length))

  (@const $Term::String::EMPTY i32 (call $Term::TermType::String::new (i32.const 0)))

  (func $Term::String::allocate (export "allocateString") (param $length i32) (result i32)
    ;; TODO: Enforce unique interned strings
    (if (result i32)
      (i32.eqz (local.get $length))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::String::EMPTY))
      (else
        (call $Term::String::allocate_sized (local.get $length)))))

  (func $Term::String::drop (param $self i32)
    ;; Avoid dropping the global empty string instance
    (if (i32.ne (local.get $self) (call $Term::String::empty))
      (then
        (call $Term::drop (local.get $self)))))

  (func $Term::String::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the string data pointer for an imaginary string term located
    ;; at memory address 0. The pointer offset tells us how many bytes are taken up by the preceding string wrapper.
    (call $Term::String::get::data::pointer (i32.const 0) (i32.const 0)))

  (func $Term::String::allocate_sized (param $length i32) (result i32)
    (local $self i32)
    ;; Allocates a new String term with the given capacity in bytes, allowing data to be copied directly into the allocated array.
    ;; The string must be instantiated before it can be used.
    ;; The standard constructor wrappers take care of allocating space for a standard term,
    ;; however they do not allocate space for extra elements as needed by the string term.
    ;; This means we have to manually allocate a larger amount of space than usual,
    ;; then fill in the string term contents into the newly-allocated space.
    ;; First allocate a new term wrapper with the required capacity
    (local.tee $self
      (call $Allocator::allocate
        (i32.add
          (call $Term::String::empty::sizeof)
          (call $Allocator::pad_to_4_byte_offset (local.get $length)))))
    ;; Then manually write the string struct contents into the term wrapper
    (call $TermType::String::construct (call $Term::pointer::value (local.get $self)) (i32.const 0))
    ;; Set the string character length
    (call $Term::String::set::length (local.get $self) (local.get $length))
    ;; Set the capacity of the data array, padded to 4-byte cell size
    (call $Term::String::set::data::capacity
      (local.get $self)
      (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $length)) (i32.const 4)))
    ;; Set the length of the data array, padded to 4-byte cell size
    (call $Term::String::set::data::length (local.get $self) (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $length)) (i32.const 4))))

  (func $Term::String::init (export "initString") (param $self i32) (result i32)
    ;; Instantiate the term
    (call $Term::init (local.get $self)))

  (func $Term::String::allocate_unsized (result i32)
    ;; Allocate a new zero-length string, ready for data to be written into its contents
    ;; The string must be instantiated before it can be used.
    (call $Term::String::allocate_sized (i32.const 0)))

  (func $Term::String::init_unsized (param $self i32) (param $length i32) (result i32)
    (local $padded_length i32)
    (local.set $padded_length (call $Allocator::pad_to_4_byte_offset (local.get $length)))
    ;; Extend the allocated space to fill the enclosing 4-byte cell
    (call $Allocator::extend
      (call $Term::String::get_char_offset (local.get $self) (local.get $length))
      (i32.sub (local.get $padded_length) (local.get $length)))
    ;; Update the capacity of the data array, padded to 4-byte cell size
    (call $Term::String::set::data::capacity
      (local.get $self)
      (i32.div_u (local.get $padded_length) (i32.const 4)))
    ;; Update the length of the data array, padded to 4-byte cell size
    (call $Term::String::set::data::length (local.get $self) (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $length)) (i32.const 4)))
    ;; Set the string character length
    (call $Term::String::set::length (local.get $self) (local.get $length))
    ;; Instantiate the term
    (call $Term::String::init (local.get $self)))

  (func $Term::String::empty (result i32)
    ;; Allocate a new string of the required length
    ;; (this will return the pre-allocated empty string singleton)
    (call $Term::String::allocate (i32.const 0)))

  (func $Term::String::from (param $value i32) (result i32)
    (local $instance i32)
    (local $end_offset i32)
    (if (result i32)
      ;; If the provided value is already a string, return it as-is
      (call $Term::String::is (local.get $value))
      (then
        (local.get $value))
      (else
        ;; Allocate a new dynamic string term
        (local.set $instance (call $Term::String::allocate_unsized))
        ;; Serialize the input term into the newly-allocated string contents
        (local.set $end_offset
          (call $Term::traits::display
            (local.get $value)
            (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0))))
        ;; Initialize the dynamic string term
        (call $Term::String::init_unsized
          (local.get $instance)
          (i32.sub
            (local.get $end_offset)
            (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))))))

  (func $Term::String::from_i32 (param $value i32) (result i32)
    (local $instance i32)
    (local $end_offset i32)
     ;; Allocate a new dynamic string term
    (local.set $instance (call $Term::String::allocate_unsized))
    (local.set $end_offset (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))
    ;; Serialize the input term into the newly-allocated string contents
    (call $Utils::i32::write_string (local.get $value) (local.get $end_offset))
    (local.set $end_offset (i32.add (local.get $end_offset)))
    ;; Initialize the dynamic string term
    (call $Term::String::init_unsized
      (local.get $instance)
      (i32.sub
        (local.get $end_offset)
        (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))))

  (func $Term::String::from_u32 (param $value i32) (result i32)
    (local $instance i32)
    (local $end_offset i32)
     ;; Allocate a new dynamic string term
    (local.set $instance (call $Term::String::allocate_unsized))
    (local.set $end_offset (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))
    ;; Serialize the input term into the newly-allocated string contents
    (call $Utils::u32::write_string (local.get $value) (local.get $end_offset))
    (local.set $end_offset (i32.add (local.get $end_offset)))
    ;; Initialize the dynamic string term
    (call $Term::String::init_unsized
      (local.get $instance)
      (i32.sub
        (local.get $end_offset)
        (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))))

  (func $Term::String::from_i64 (param $value i64) (result i32)
    (local $instance i32)
    (local $end_offset i32)
     ;; Allocate a new dynamic string term
    (local.set $instance (call $Term::String::allocate_unsized))
    (local.set $end_offset (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))
    ;; Serialize the input term into the newly-allocated string contents
    (call $Utils::i64::write_string (local.get $value) (local.get $end_offset))
    (local.set $end_offset (i32.add (local.get $end_offset)))
    ;; Initialize the dynamic string term
    (call $Term::String::init_unsized
      (local.get $instance)
      (i32.sub
        (local.get $end_offset)
        (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))))

  (func $Term::String::from_u64 (param $value i64) (result i32)
    (local $instance i32)
    (local $end_offset i32)
     ;; Allocate a new dynamic string term
    (local.set $instance (call $Term::String::allocate_unsized))
    (local.set $end_offset (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))
    ;; Serialize the input term into the newly-allocated string contents
    (call $Utils::u64::write_string (local.get $value) (local.get $end_offset))
    (local.set $end_offset (i32.add (local.get $end_offset)))
    ;; Initialize the dynamic string term
    (call $Term::String::init_unsized
      (local.get $instance)
      (i32.sub
        (local.get $end_offset)
        (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)))))

  (func $Term::String::from_char (param $char i32) (result i32)
    (local $instance i32)
    ;; Allocate a new String term with the correct capacity
    (local.tee $instance (call $Term::String::allocate (i32.const 1)))
    ;; Copy the character into the data array
    (i32.store8 (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0)) (local.get $char))
    ;; Instantiate the term
    (call $Term::String::init))

  (func $Term::String::from_slice (param $offset i32) (param $length i32) (result i32)
    (local $self i32)
    ;; Allocates a new String term whose contents is copied from the given slice of linear memory
    (if (result i32)
      (i32.eqz (local.get $length))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::String::EMPTY))
      (else
        ;; Allocate a new String term with the correct capacity
        (local.tee $self (call $Term::String::allocate (local.get $length)))
        ;; Copy the slice into the data array
        (memory.copy
          (call $Term::String::get::data::pointer (local.get $self) (i32.const 0))
          (local.get $offset)
          (local.get $length))
        ;; Instantiate the term
        (call $Term::String::init))))

  (func $Term::String::copy_contents (param $self i32) (param $target_offset i32) (result i32)
    (local $length i32)
    ;; Copy the slice into the target memory address
    (memory.copy
      (local.get $target_offset)
      (call $Term::String::get::data::pointer (local.get $self) (i32.const 0))
      (local.tee $length (call $Term::String::get_length (local.get $self))))
    ;; Return the number of bytes written
    (local.get $length))

  (func $Term::String::get_offset (export "getStringOffset") (param $self i32) (result i32)
    (call $Term::String::get::data::pointer (local.get $self) (i32.const 0)))

  (func $Term::String::get_length (param $self i32) (result i32)
    (call $Term::String::get::length (local.get $self)))

  (func $Term::String::get_char_offset (export "getStringCharOffset") (param $self i32) (param $index i32) (result i32)
    (i32.add
      (call $Term::String::get_offset (local.get $self))
      (local.get $index)))

  (func $Term::String::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::String::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::String::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $length i32)
    (call $Allocator::extend
      (local.get $offset)
      (local.tee $length (call $Term::String::get_length (local.get $self))))
    (memory.copy
      (local.get $offset)
      (call $Term::String::get_offset (local.get $self))
      (local.get $length))
    (i32.add (local.get $offset) (local.get $length)))

  (func $Term::String::traits::debug (param $self i32) (param $offset i32) (result i32)
    ;; Write the serialized value to the output string and return the updated offset
    (i32.add
      (local.get $offset)
      (call $Utils::bytes::write_json
        (call $Term::String::get_offset (local.get $self))
        (call $Term::String::get_length (local.get $self))
        (local.get $offset))))

  (func $Term::String::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::String::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Put the success marker on the stack
    (global.get $TRUE)
    ;; Write the serialized value to the output string and return the updated offset
    (i32.add
      (local.get $offset)
      (call $Utils::bytes::write_json
        (call $Term::String::get_offset (local.get $self))
        (call $Term::String::get_length (local.get $self))
        (local.get $offset))))

  (func $Term::String::traits::length (param $self i32) (result i32)
    (call $Term::String::get_length (local.get $self)))

  (func $Term::String::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $values i32)
    (local $dependencies i32)
    ;; Collect the iterator into a temporary list instance
    ;; TODO: Avoid unnecessary heap allocations for intermediate values
    (call $Term::List::traits::collect (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (local.tee $values)
    (call $Term::String::collect_string_list)
    ;; Dispose the temporary list instance
    (call $Term::drop (local.get $values))
    (local.get $dependencies))

  (func $Term::String::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $values i32)
    (local $dependencies i32)
    ;; Collect the iterator into a temporary list instance
    ;; TODO: Avoid unnecessary heap allocations for intermediate values
    (call $Term::List::traits::collect_strict (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (local.tee $values)
    (call $Term::String::collect_string_list)
    ;; Dispose the temporary list instance
    (call $Term::drop (local.get $values))
    (local.get $dependencies))

  (func $Term::String::collect_string_list (param $sources i32) (result i32)
    (local $source_count i32)
    (local $source_index i32)
    (local $length i32)
    (local $instance i32)
    (local $target_offset i32)
    (local $source i32)
    (local $source_length i32)
    (local $type_error i32)
    (local.set $source_count (call $Term::List::get_length (local.get $sources)))
    (if (result i32)
      ;; If no source strings were provided, return the empty string
      (i32.eqz (local.get $source_count))
      (then
        (call $Term::String::empty))
      (else
        ;; Otherwise compute the combined length of all the source strings, returning a type error
        ;; if any non-strings were encountered
        (local.set $type_error (global.get $NULL))
        (loop $LOOP
          (if
            ;; If the current item is not a string, update the combined error
            (i32.eqz
              (call $Term::String::is
                (local.tee $source (call $Term::List::get_item (local.get $sources) (local.get $source_index)))))
            (then
              (local.set $type_error
                (call $Term::Signal::traits::union
                  (local.get $type_error)
                  (call $Term::Signal::of
                    (call $Term::Condition::type_error (global.get $TermType::String) (local.get $source))))))
            (else
              ;; Otherwise increase the combined string length
              (local.set $length (i32.add (local.get $length) (call $Term::String::get_length (local.get $source))))))
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
              (return (call $Term::String::empty))))
          ;; Otherwise allocate a string to hold the combined result
          (local.tee $instance (call $Term::String::allocate (local.get $length)))
          ;; Copy the source string contents into the result string
          (local.set $source_index (i32.const 0))
          (local.set $length (i32.const 0))
          (local.set $target_offset (call $Term::String::get_offset (local.get $instance)))
          (loop $LOOP
            (memory.copy
              (i32.add (local.get $target_offset) (local.get $length))
              (call $Term::String::get_offset (local.tee $source (call $Term::List::get_item (local.get $sources) (local.get $source_index))))
              (local.tee $source_length (call $Term::String::get_length (local.get $source))))
            (local.set $length (i32.add (local.get $length) (local.get $source_length)))
            (br_if $LOOP (i32.lt_u (local.tee $source_index (i32.add (local.get $source_index) (i32.const 1))) (local.get $source_count))))
          ;; Initialize the string term
          (call $Term::String::init)))))

  (func $Term::String::get_char_pointer (param $self i32) (param $index i32) (result i32)
    (i32.add
      (call $String::get::data::pointer
        (call $Term::get_value (local.get $self))
        (i32.const 0))
      (local.get $index)))

  (func $Term::String::get_char (param $self i32) (param $index i32) (result i32)
    (call $Term::String::from_char (i32.load8_u (call $Term::String::get_char_pointer (local.get $self) (local.get $index)))))

  (func $Term::String::slice (param $self i32) (param $offset i32) (param $length i32) (result i32)
    (local $instance i32)
    (local $source_length i32)
    (if (result i32)
      ;; If the specified region encompasses the whole string, return the unmodified string
      (i32.and
        (i32.eqz (local.get $offset))
        (i32.ge_u (local.get $length) (local.tee $source_length (call $Term::String::get_length (local.get $self)))))
      (then
        (local.get $self))
      (else
        ;; Otherwise if the specified offset is beyond the end of the string, or if the length is zero, return the empty string
        (if (result i32)
          (i32.or
            (i32.ge_u (local.get $offset) (local.get $source_length))
            (i32.eqz (local.get $length)))
          (then
            (call $Term::String::empty))
          (else
            ;; Otherwise allocate a new string of the correct length
            (local.tee $instance
              (call $Term::String::allocate
                (local.tee $length
                  (call $Utils::i32::min_u
                    (local.get $length)
                    (i32.sub (local.get $source_length) (local.get $offset))))))
            ;; Copy the existing contents into the new string
            (memory.copy
              (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0))
              (call $Term::String::get_char_pointer (local.get $self) (local.get $offset))
              (local.get $length))
            ;; Instantiate the new string
            (call $Term::String::init))))))

  (func $Term::String::find_index (param $self i32) (param $offset i32) (param $pattern_offset i32) (param $pattern_length i32) (result i32)
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
          (i32.gt_u (local.get $pattern_length) (local.tee $length (call $Term::String::get_length (local.get $self))))
          (then
            (global.get $NULL))
          (else
            ;; Otherwise iterate through the string contents looking for a match
            (local.set $offset (call $Term::String::get_char_pointer (local.get $self) (local.get $offset)))
            (local.set $max_offset (call $Term::String::get_char_pointer (local.get $self) (i32.sub (local.get $length) (local.get $pattern_length))))
            (loop $LOOP
              ;; If the pattern exists at the current index, return the current index
              (if
                (call $Utils::bytes::equals
                  (local.get $offset)
                  (local.get $pattern_length)
                  (local.get $pattern_offset)
                  (local.get $pattern_length))
                (then
                  (return (i32.sub (local.get $offset) (call $Term::String::get_offset (local.get $self)))))
                (else
                  ;; Otherwise continue from the next character until the entire string has been iterated
                  (br_if $LOOP
                    (i32.le_u
                      (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))
                      (local.get $max_offset))))))
            ;; The entire string has been iterated without finding a match; return the null sentinel value
            (global.get $NULL))))))

  (func $Term::String::replace (param $self i32) (param $pattern_offset i32) (param $pattern_length i32) (param $replacement_offset i32) (param $replacement_length i32) (result i32)
    (local $match_index i32)
    (local $instance i32)
    (local $input_length i32)
    (local $output_length i32)
    ;; Find the first index of the given pattern
    (if (result i32)
      (i32.eq (global.get $NULL) (local.tee $match_index (call $Term::String::find_index (local.get $self) (i32.const 0) (local.get $pattern_offset) (local.get $pattern_length))))
      (then
        ;; If the pattern was not found, return the unmodified string
        (local.get $self))
      (else
        ;; Otherwise create a new string with the pattern replaced
        (local.set $output_length
          (i32.add
            (local.tee $input_length (call $Term::String::get_length (local.get $self)))
            (i32.sub (local.get $replacement_length) (local.get $pattern_length))))
        ;; If the replacement is the same length as the original pattern, create a copy of the input string
        (if (result i32)
          (i32.eq (local.get $output_length) (local.get $input_length))
          (then
            (if (result i32)
              (i32.eqz (local.get $replacement_length))
              (then
                ;; If the replacement is the empty string, return the unmodified term
                (local.get $self))
              (else
                ;; Otherwise if the replacement is the same length as the original pattern, create a clone of the input string
                (local.tee $instance (call $Term::traits::clone (local.get $self)))
                ;; Copy the replacement string into the new string
                (memory.copy
                  (call $Term::String::get_char_pointer (local.get $instance) (local.get $match_index))
                  (local.get $replacement_offset)
                  (local.get $replacement_length))
                ;; Instantiate the term
                (call $Term::String::init))))
          (else
            ;; Otherwise allocate a new string with the correct length
            (local.tee $instance
              (call $Term::String::allocate (local.get $output_length)))
            ;; Copy the string contents before the pattern match into the output string
            (memory.copy
              (call $Term::String::get_offset (local.get $instance))
              (call $Term::String::get_offset (local.get $self))
              (local.get $match_index))
            ;; Copy the replacement string into the new string
            (memory.copy
              (call $Term::String::get_char_pointer (local.get $instance) (local.get $match_index))
              (local.get $replacement_offset)
              (local.get $replacement_length))
            ;; Copy the string contents after the pattern match into the output string
            (memory.copy
              (call $Term::String::get_char_pointer
                (local.get $instance)
                (i32.add (local.get $match_index) (local.get $replacement_length)))
              (call $Term::String::get_char_pointer
                (local.get $self)
                (i32.add (local.get $match_index) (local.get $pattern_length)))
              (i32.sub
                (local.get $input_length)
                (i32.add (local.get $match_index) (local.get $pattern_length))))
            ;; Instantiate the term
            (call $Term::String::init))))))

  (func $Term::String::split (param $self i32) (param $pattern_offset i32) (param $pattern_length i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local $previous_offset i32)
    (local $results i32)
    (local $num_matches i32)
    (if (result i32)
      ;; If the source string is empty, no need to test for matches
      (i32.eqz (local.tee $length (call $Term::String::get_length (local.get $self))))
      (then
        (if (result i32)
          (i32.eqz (local.get $pattern_length))
          (then
            ;; If both the source string and the search pattern are empty, return the empty list
            (call $Term::List::empty))
          (else
            ;; Otherwise return a 1-item list containing the empty string
            (call $Term::List::of (call $Term::String::empty)))))
      (else
        ;; Otherwise if the pattern is the empty string, return a list of the string characters
        (if (result i32)
          (i32.eqz (local.get $pattern_length))
          (then
            ;; Allocate a new list of the correct length
            (local.tee $results (call $Term::List::allocate (local.get $length)))
            ;; Iterate through the source string adding each character to the list
            (loop $LOOP
              (call $Term::List::set_item
                (local.get $results)
                (local.get $index)
                (call $Term::String::from_char (i32.load8_u (call $Term::String::get_char_pointer (local.get $self) (local.get $index)))))
              ;; Continue with the next character until the end of the source string is reached
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $length))))
            ;; Instantiate the new list
            (call $Term::List::init (local.get $length)))
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
            (local.set $results (call $Term::List::allocate_unsized))
            ;; Iterate through the source string searching for matches, and store them in the results list
            (loop $LOOP
              (if
                ;; If the pattern was not found, we have reached the end of the string
                (i32.eq
                  (global.get $NULL)
                  (local.tee $index
                    (call $Term::String::find_index
                      (local.get $self)
                      (local.get $index)
                      (local.get $pattern_offset)
                      (local.get $pattern_length))))
                (then)
                (else
                  ;; Otherwise store the match offset in the results list
                  (local.set $results (call $Term::List::grow_unsized (local.get $results) (local.get $index)))
                  ;; If we have not yet reached the end of the source string, search for the next match
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (local.get $index) (local.get $pattern_length)))
                      (local.get $length))))))
            (if (result i32)
              (i32.eqz (local.tee $num_matches (call $Term::List::get_length (local.get $results))))
              (then
                ;; If there were no matches, return a list containing the whole unmodified source string
                (call $Term::List::of (local.get $self)))
              (else
                ;; Append a final substring that spans to the end of the string
                (local.set $results (call $Term::List::grow_unsized (local.get $results) (local.get $length)))
                ;; Now that we have found all the matches, iterate through the results
                ;; creating substrings for each of the matches and overwring the contents of the results list
                (local.set $index (i32.const 0))
                (loop $LOOP
                  ;; Overwrite the current list item
                  (call $Term::List::set_item
                    (local.get $results)
                    (local.get $index)
                    ;; Calculate offset and length values for the current substring
                    (call $Term::String::create_slice_from_start_end_offsets
                      ;; Start the substring at the end of the previous match, or zero if this is the first iteration
                      (call $Term::String::get_char_pointer
                        (local.get $self)
                        (select
                          (i32.const 0)
                          (i32.add (local.get $previous_offset) (local.get $pattern_length))
                          (i32.eqz (local.get $index))))
                      ;; End the substring at the address of the next match (retrieved from the results list)
                      (call $Term::String::get_char_pointer
                        (local.get $self)
                        (local.tee $previous_offset
                          ;; Load the integer value of the result offset from the results list
                          (i32.load (call $Term::List::get::items::pointer (local.get $results) (local.get $index))))))
                    ;; Allocate a substring with the given offset and length
                    (call $Term::String::from_slice))
                  ;; Advance to the next match
                  (br_if $LOOP
                    (i32.le_u
                      (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                      (local.get $num_matches))))
                ;; Instantiate the list of substrings
                (local.get $results)
                (call $Term::List::init_unsized))))))))

  (func $Term::String::create_slice_from_start_end_offsets (param $start i32) (param $end i32) (result i32 i32)
    (local.get $start)
    (i32.sub (local.get $end) (local.get $start))))
