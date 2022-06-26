;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Urlencode "Urlencode"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Urlencode::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local $offset i32)
        (local $length i32)
        (local $index i32)
        (local $output i32)
        (if (result i32 i32)
          (i32.eq
            (local.tee $index
              (call $Stdlib_Urlencode::find_next_special_char_index
                (local.tee $offset (call $Term::String::get_char_pointer (local.get $self) (i32.const 0)))
                (local.tee $length (call $Term::String::get_length (local.get $self)))))
            (global.get $NULL))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            (local.tee $output (call $Term::String::allocate_unsized))
            (local.set $output (call $Term::String::get_char_pointer (local.get $output) (i32.const 0)))
            (call $Allocator::extend (local.get $output) (local.get $index))
            (memory.copy (local.get $output) (local.get $offset) (local.get $index))
            (call $Stdlib_Urlencode::write_escaped_bytes
              (i32.add (local.get $offset) (local.get $index))
              (i32.sub (local.get $length) (local.get $index))
              (i32.add (local.get $output) (local.get $index)))
            (i32.add (local.get $index))
            (call $Term::String::init_unsized)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Urlencode::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Urlencode)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL))))

  (func $Stdlib_Urlencode::find_next_special_char_index (param $offset i32) (param $length i32) (result i32)
    (local $index i32)
    (if (result i32)
      (i32.eqz (local.get $length))
      (then
        (global.get $NULL))
      (else
        (loop $LOOP
          (if
            (call $Stdlib_Urlencode::is_special_char
              (i32.load8_u (i32.add (local.get $offset) (local.get $index))))
            (then
              (return (local.get $index)))
            (else
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        (global.get $NULL))))

  (func $Stdlib_Urlencode::is_special_char (param $char i32) (result i32)
    (i32.eq (local.get $char) (@char " "))
    (i32.eq (local.get $char) (@char "#"))
    (i32.eq (local.get $char) (@char "$"))
    (i32.eq (local.get $char) (@char "%"))
    (i32.eq (local.get $char) (@char "&"))
    (i32.eq (local.get $char) (@char "+"))
    (i32.eq (local.get $char) (@char ","))
    (i32.eq (local.get $char) (@char "/"))
    (i32.eq (local.get $char) (@char ":"))
    (i32.eq (local.get $char) (@char ";"))
    (i32.eq (local.get $char) (@char "="))
    (i32.eq (local.get $char) (@char "?"))
    (i32.eq (local.get $char) (@char "@"))
    (i32.eq (local.get $char) (@char "["))
    (i32.eq (local.get $char) (@char "]"))
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or)
    (i32.or))

  (func $Stdlib_Urlencode::get_encoded_special_char_bytes (param $char i32) (result i32)
    ;; Given an ASCII byte, return the corresponding two bytes to use as the numeric portion of the
    ;; percent-escaped escape sequence, or zero if this character does not need escaping
    (@switch
      (@list
        (@list
          (i32.eq (local.get $char) (@char " "))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "0"))))
        (@list
          (i32.eq (local.get $char) (@char "#"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "3"))))
        (@list
          (i32.eq (local.get $char) (@char "$"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "4"))))
        (@list
          (i32.eq (local.get $char) (@char "%"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "5"))))
        (@list
          (i32.eq (local.get $char) (@char "&"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "6"))))
        (@list
          (i32.eq (local.get $char) (@char "+"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "B"))))
        (@list
          (i32.eq (local.get $char) (@char ","))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "C"))))
        (@list
          (i32.eq (local.get $char) (@char "/"))
          (return (i32.or (i32.shl (@char "2") (i32.const 8)) (@char "F"))))
        (@list
          (i32.eq (local.get $char) (@char ":"))
          (return (i32.or (i32.shl (@char "3") (i32.const 8)) (@char "A"))))
        (@list
          (i32.eq (local.get $char) (@char ";"))
          (return (i32.or (i32.shl (@char "3") (i32.const 8)) (@char "B"))))
        (@list
          (i32.eq (local.get $char) (@char "="))
          (return (i32.or (i32.shl (@char "3") (i32.const 8)) (@char "D"))))
        (@list
          (i32.eq (local.get $char) (@char "?"))
          (return (i32.or (i32.shl (@char "3") (i32.const 8)) (@char "F"))))
        (@list
          (i32.eq (local.get $char) (@char "@"))
          (return (i32.or (i32.shl (@char "4") (i32.const 8)) (@char "0"))))
        (@list
          (i32.eq (local.get $char) (@char "["))
          (return (i32.or (i32.shl (@char "5") (i32.const 8)) (@char "B"))))
        (@list
          (i32.eq (local.get $char) (@char "]"))
          (return (i32.or (i32.shl (@char "5") (i32.const 8)) (@char "D")))))
      ;; Default case
      (i32.const 0)))

  (func $Stdlib_Urlencode::write_escaped_bytes (param $source_offset i32) (param $source_length i32) (param $target_offset i32) (result i32)
    (local $original_offset i32)
    (local $index i32)
    (local $special_char i32)
    (if (result i32)
      ;; If the input string is empty, bail out
      (i32.eqz (local.get $source_length))
      (then
        ;; Return the number of bytes written
        (i32.const 0))
      (else
        ;; Keep track of the original offset to work out the number of bytes written
        (local.set $original_offset (local.get $target_offset))
        ;; Scan through the source string to detect any special characters that need escaping
        (loop $LOOP
          ;; Scan the source string until we encounter either the end of the string or a special character
          (loop $INNER
            ;; Increment the index scanning one byte at a time until either of these conditions is hit
            (br_if $INNER
              (i32.and
                ;; Note that both conditions are evaluated within the same expression to save on branching within the loop.
                ;; This means that in the case where we stop at a special character, the index will be
                ;; incremented regardless (hence the need to decrement the index later).
                (i32.eqz
                  (local.tee $special_char
                    (call $Stdlib_Urlencode::is_special_char
                      (i32.load8_u (i32.add (local.get $source_offset) (local.get $index))))))
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $source_length)))))
          ;; If this is a special character, decrement the index to counteract the superfluous addition within the loop
          (local.set $index (i32.sub (local.get $index) (local.get $special_char)))
          ;; Allocate enough space to store the chunk, plus the escape sequence if applicable
          (call $Allocator::extend
            (local.get $target_offset)
            (select
              ;; Special characters need three bytes to store the percent-escaped sequence
              (i32.add (local.get $index) (i32.const 3))
              (local.get $index)
              (local.get $special_char)))
          ;; Copy the chunk into the output string
          (memory.copy
            (local.get $target_offset)
            (local.get $source_offset)
            (local.get $index))
          ;; Update the current offset to reflect the addition of the chunk
          (local.set $target_offset (i32.add (local.get $target_offset) (local.get $index)))
          (if
            ;; If we stopped at a special character, write its escaped form into the output
            (local.get $special_char)
            (then
              ;; Write a percent character to the output
              (i32.store8 offset=0 (local.get $target_offset) (@char "%"))
              ;; Retrieve the correct escape code bytes and write them to the output
              (local.set $special_char
                (call $Stdlib_Urlencode::get_encoded_special_char_bytes
                  (i32.load8_u (i32.add (local.get $source_offset) (local.get $index)))))
              (i32.store8 offset=1 (local.get $target_offset) (i32.and (i32.shr_u (local.get $special_char) (i32.const 8)) (i32.const 0xFF)))
              (i32.store8 offset=2 (local.get $target_offset) (i32.and (local.get $special_char) (i32.const 0xFF)))
              ;; Update the target offset to reflect the addition of the escape sequence
              (local.set $target_offset (i32.add (local.get $target_offset) (i32.const 3)))
              ;; Update the source offset and remaining length, skipping over the escaped character
              (local.set $source_offset (i32.add (local.get $source_offset) (local.tee $index (i32.add (local.get $index) (i32.const 1)))))
              (local.tee $source_length (i32.sub (local.get $source_length) (local.get $index)))
              (local.set $index (i32.const 0))
              ;; If the remaining length is non-zero, continue with the next chunk
              (br_if $LOOP))
            (else)))
          ;; Return the final number of bytes written
          (i32.sub (local.get $target_offset) (local.get $original_offset))))))
