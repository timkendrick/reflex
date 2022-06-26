;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Json::parse (param $offset i32) (param $length i32) (result i32 i32)
    (local $result i32)
    ;; Conver the length to an end offset
    (local.set $length (i32.add (local.get $offset) (local.get $length)))
    ;; Parse the Json value
    (call $Json::parse_value (local.get $offset) (local.get $length))
    (local.set $offset)
    (local.set $result)
    (if (result i32 i32)
      ;; If all the the input has been parsed, or if the input was not valid Json, return the parse result
      (i32.or
        (i32.eq (local.get $offset) (local.get $length))
        (i32.eq (global.get $NULL) (local.get $result)))
      (then
        (local.get $result)
        (local.get $offset))
      (else
        ;; Skip any trailing whitespace
        (call $Json::skip_whitespace (local.get $offset) (local.get $length))
        ;; Discard the lookahead character
        (drop)
        (local.set $offset)
        (if (result i32 i32)
          ;; If the end of the input has been reached, return the result
          (i32.eq (local.get $offset) (local.get $length))
          (then
            (local.get $result)
            (local.get $offset))
          (else
            ;; Otherwise there must be trailing non-whitespace characters, so return an error result
            (call $Json::parse_error (local.get $offset)))))))

  (func $Json::parse_value (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    ;; Skip through any whitespace characters
    (call $Json::skip_whitespace (local.get $offset) (local.get $end_offset))
    (local.set $char)
    (local.set $offset)
    ;; If we have reached the end of the source string, return an error result
    (if (result i32 i32)
      (i32.eq (local.get $offset) (local.get $end_offset))
      (then
        (call $Json::parse_error (local.get $offset)))
      (else
        ;; Otherwise determine the correct parser depending on the current character
        (block $default
          (block $object
            (block $array
              (block $string
                (block $number
                  (block $true
                    (block $false
                      (block $null
                        (br_table
                          $null
                          $false
                          $true
                          $number
                          $string
                          $array
                          $object
                          $default
                          (select
                            (i32.const 0)
                            (select
                              (i32.const 1)
                              (select
                                (i32.const 2)
                                (select
                                  (i32.const 3)
                                  (select
                                    (i32.const 4)
                                    (select
                                      (i32.const 5)
                                      (select
                                        (i32.const 6)
                                        (i32.const 7)
                                        (i32.eq (@char "{") (local.get $char)))
                                      (i32.eq (@char "[") (local.get $char)))
                                    (i32.eq (@char "\"") (local.get $char)))
                                  (i32.or (i32.eq (@char "-") (local.get $char)) (call $Json::is_digit (local.get $char))))
                                (i32.eq (@char "t") (local.get $char)))
                              (i32.eq (@char "f") (local.get $char)))
                            (i32.eq (@char "n") (local.get $char)))))
                        (return (call $Json::parse_null (local.get $offset) (local.get $end_offset))))
                      (return (call $Json::parse_false (local.get $offset) (local.get $end_offset))))
                    (return (call $Json::parse_true (local.get $offset) (local.get $end_offset))))
                  (return (call $Json::parse_number (local.get $offset) (local.get $end_offset))))
                (return (call $Json::parse_string (local.get $offset) (local.get $end_offset))))
              (return (call $Json::parse_array (local.get $offset) (local.get $end_offset))))
            (return (call $Json::parse_object (local.get $offset) (local.get $end_offset))))
          ;; If an invalid Json value was provided, return a parse error
          (call $Json::parse_error (local.get $offset)))))

  (func $Json::parse_error (param $offset i32) (result i32 i32)
    ;; Return the null sentinel value and the current offset
    (global.get $NULL)
    (local.get $offset))

  (func $Json::skip_whitespace (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    (if (result i32 i32)
      (i32.eq (local.get $offset) (local.get $end_offset))
      (then
        (local.get $offset)
        (global.get $NULL))
      (else
        (loop $LOOP
          (if
            (call $Json::is_whitespace (local.tee $char (i32.load8_u (local.get $offset))))
            (then
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))
                  (local.get $end_offset))))
            (else)))
        (local.get $offset)
        (local.get $char))))

  (func $Json::is_whitespace (param $char i32) (result i32)
    (i32.or
      (i32.eq (local.get $char) (@char " "))
      (i32.or
        (i32.eq (local.get $char) (@char "\n"))
        (i32.or
          (i32.eq (local.get $char) (@char "\r"))
          (i32.eq (local.get $char) (@char "\t"))))))

  (func $Json::parse_null (param $offset i32) (param $end_offset i32) (result i32 i32)
    (if (result i32 i32)
      (if (result i32)
        (i32.lt_u (local.get $end_offset) (i32.add (local.get $offset) (i32.const 4)))
        (then
          (i32.const 0))
        (else
          (i32.and
            (i32.eq (@char "u") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
            (i32.and
              (i32.eq (@char "l") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
              (i32.eq (@char "l") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))))))
      (then
        (call $Term::Nil::new)
        (i32.add (local.get $offset) (i32.const 1)))
      (else
        (call $Json::parse_error (local.get $offset)))))

  (func $Json::parse_false (param $offset i32) (param $end_offset i32) (result i32 i32)
    (if (result i32 i32)
      (if (result i32)
        (i32.lt_u (local.get $end_offset) (i32.add (local.get $offset) (i32.const 5)))
        (then
          (i32.const 0))
        (else
          (i32.and
            (i32.eq (@char "a") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
            (i32.and
              (i32.eq (@char "l") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
              (i32.and
                (i32.eq (@char "s") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
                (i32.eq (@char "e") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1))))))))))
      (then
        (call $Term::Boolean::false)
        (i32.add (local.get $offset) (i32.const 1)))
      (else
        (call $Json::parse_error (local.get $offset)))))

  (func $Json::parse_true (param $offset i32) (param $end_offset i32) (result i32 i32)
    (if (result i32 i32)
      (if (result i32)
        (i32.lt_u (local.get $end_offset) (i32.add (local.get $offset) (i32.const 4)))
        (then
          (i32.const 0))
        (else
          (i32.and
            (i32.eq (@char "r") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
            (i32.and
              (i32.eq (@char "u") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))
              (i32.eq (@char "e") (i32.load8_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1)))))))))
      (then
        (call $Term::Boolean::true)
        (i32.add (local.get $offset) (i32.const 1)))
      (else
        (call $Json::parse_error (local.get $offset)))))

  (func $Json::parse_number (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    (local $is_negative i32)
    (local $integer_value i32)
    (local $decimal_value i32)
    (local $decimal_precision i32)
    (local $exponent_value i32)
    (local $is_negative_exponent i32)
    (local.set $is_negative (i32.eq (@char "-") (local.tee $char (i32.load8_u (local.get $offset)))))
    (local.set $offset (i32.add (local.get $offset) (local.get $is_negative)))
    (if (result i32 i32)
      (i32.eq (local.get $offset) (local.get $end_offset))
      (then
        (call $Json::parse_error (local.get $offset)))
      (else
        (loop $LOOP
          (if
            (call $Json::is_digit (local.tee $char (i32.load8_u (local.get $offset))))
            (then
              (local.set $integer_value
                (i32.add
                  (i32.mul (local.get $integer_value) (i32.const 10))
                  (call $Json::parse_digit (local.get $char))))
              (br_if $LOOP (i32.lt_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1))) (local.get $end_offset))))
            (else)))
        (block $default
          (block $exponent
            (block $decimal
              (br_table
                $decimal
                $exponent
                $default
                (select
                  (i32.const 0)
                  (select
                    (i32.const 1)
                    (i32.const 2)
                    (call $Json::is_exponent_signifier (local.get $char)))
                  (i32.eq (local.get $char) (@char ".")))))
            (; decimal ;)
            (if
              (i32.eq (local.tee $offset (i32.add (local.get $offset) (i32.const 1))) (local.get $end_offset))
              (then
                (return (call $Json::parse_error (local.get $offset))))
              (else
                (loop $LOOP
                  (if
                    (call $Json::is_digit (local.tee $char (i32.load8_u (local.get $offset))))
                    (then
                      (local.set $decimal_precision (i32.add (local.get $decimal_precision) (i32.const 1)))
                      (local.set $decimal_value
                        (i32.add
                          (i32.mul (local.get $decimal_value) (i32.const 10))
                          (call $Json::parse_digit (local.get $char))))
                      (br_if $LOOP (i32.lt_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1))) (local.get $end_offset))))
                    (else)))
                (if
                  (i32.eqz (local.get $decimal_precision))
                  (then
                    (return (call $Json::parse_error (local.get $offset))))
                  (else
                    (return
                      (call $Term::Float::new
                        (call $Json::parse_float
                          (local.get $is_negative)
                          (local.get $integer_value)
                          (local.get $decimal_value)
                          (local.get $decimal_precision)
                          (if (result i32)
                            (call $Json::is_exponent_signifier (local.get $char))
                            (then
                              (call $Json::parse_exponent (i32.add (local.get $offset) (i32.const 1)) (local.get $end_offset))
                              (local.set $offset)
                              (local.set $exponent_value)
                              (local.set $is_negative_exponent)
                              (if (result i32)
                                (i32.eq (local.get $exponent_value) (global.get $NULL))
                                (then
                                  (return (call $Json::parse_error (local.get $offset))))
                                (else
                                  (call $Json::set_negative (local.get $exponent_value) (local.get $is_negative_exponent)))))
                            (else
                              (i32.const 0)))))
                      (local.get $offset)))))))
          (; exponent ;)
          (call $Json::parse_exponent (i32.add (local.get $offset) (i32.const 1)) (local.get $end_offset))
          (local.set $offset)
          (local.set $exponent_value)
          (local.set $is_negative_exponent)
          (if
            (i32.eq (local.get $exponent_value) (global.get $NULL))
            (then
              (return (call $Json::parse_error (local.get $offset))))
            (else
              (return
                (call $Term::Float::new
                  (call $Json::parse_float
                    (local.get $is_negative)
                    (local.get $integer_value)
                    (i32.const 0)
                    (i32.const 0)
                    (call $Json::set_negative (local.get $exponent_value) (local.get $is_negative_exponent))))
                  (local.get $offset)))))
        ;; Default implementation
        (call $Term::Int::new (call $Json::set_negative (local.get $integer_value) (local.get $is_negative)))
        (local.get $offset))))

  (func $Json::is_digit (param $char i32) (result i32)
    (i32.and
      (i32.ge_u (local.get $char) (@char "0"))
      (i32.le_u (local.get $char) (@char "9"))))

  (func $Json::parse_digit (param $char i32) (result i32)
    (i32.sub (local.get $char) (@char "0")))

  (func $Json::is_exponent_signifier (param $char i32) (result i32)
    (i32.or
      (i32.eq (local.get $char) (@char "E"))
      (i32.eq (local.get $char) (@char "e"))))

  (func $Json::parse_exponent (param $offset i32) (param $end_offset i32) (result i32 i32 i32)
    (local $char i32)
    (local $is_negative i32)
    (local $exponent_value i32)
    (if (result i32 i32 i32)
      (i32.eq (local.get $offset) (local.get $end_offset))
      (then
        (global.get $NULL)
        (call $Json::parse_error (local.get $offset)))
      (else
        (local.set $char (i32.load8_u (local.get $offset)))
        (local.set $char
          (select
            (i32.const 1)
            (select
              (i32.const -1)
              (i32.const 0)
              (i32.eq (local.get $char) (@char "-")))
            (i32.eq (local.get $char) (@char "+"))))
        (local.set $is_negative (i32.eq (local.get $char) (i32.const -1)))
        (local.set $offset (i32.add (local.get $offset) (i32.ne (local.get $char) (i32.const 0))))
        (if (result i32 i32 i32)
          (i32.eq (local.get $offset) (local.get $end_offset))
          (then
            (global.get $NULL)
            (call $Json::parse_error (local.get $offset)))
          (else
            (loop $LOOP
              (if
                (call $Json::is_digit (local.tee $char (i32.load8_u (local.get $offset))))
                (then
                  (local.set $exponent_value
                    (i32.add
                      (i32.mul (local.get $exponent_value) (i32.const 10))
                      (call $Json::parse_digit (local.get $char))))
                  (br_if $LOOP (i32.lt_u (local.tee $offset (i32.add (local.get $offset) (i32.const 1))) (local.get $end_offset))))
                (else)))
            (local.get $is_negative)
            (local.get $exponent_value)
            (local.get $offset))))))

  (func $Json::set_negative (param $value i32) (param $is_negative i32) (result i32)
    (select
      (call $Utils::i32::neg (local.get $value))
      (local.get $value)
      (local.get $is_negative)))

  (func $Json::parse_float (param $is_negative i32) (param $integer_value i32) (param $decimal_value i32) (param $decimal_precision i32) (param $exponent_value i32) (result f64)
    (local $value f64)
    (local.set $value
      (f64.add
        (f64.convert_i32_s (local.get $integer_value))
        (f64.div
          (f64.convert_i32_u (local.get $decimal_value))
          (call $Utils::f64::pow_int (f64.const 10) (local.get $decimal_precision)))))
    (if
      (local.get $exponent_value)
      (then
        (local.set $value
          (f64.mul
            (local.get $value)
            (call $Utils::f64::pow_int (f64.const 10) (local.get $exponent_value)))))
      (else))
    (select
      (call $Utils::f64::neg (local.get $value))
      (local.get $value)
      (local.get $is_negative)))

  (func $Json::parse_string (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $chunk_offset i32)
    (local $result i32)
    (local $char i32)
    (local $bytes_written i32)
    (local $escape_sequence_length i32)
    (local.set $result (call $Term::String::allocate_unsized))
    ;; Store the address of the first byte of the string contents (skipping over the opening quote)
    (local.set $chunk_offset (local.tee $offset (i32.add (local.get $offset) (i32.const 1))))
    (loop $LOOP (result i32 i32)
      (if (result i32 i32)
        ;; If we have reached the end of the source string without a closing quote, bail out
        (i32.eq (local.get $offset) (local.get $end_offset))
        (then
          ;; Terminate the output string
          (call $Term::String::init (local.get $result) (local.get $bytes_written))
          (drop)
          ;; Return an error result
          (call $Json::parse_error (local.get $offset)))
        (else
          ;; Determine the implementation based on whether the current character is a special character
          (block $default
            (block $BACKSLASH
              (block $DOUBLE_QUOTE
                (br_table
                  $DOUBLE_QUOTE
                  $BACKSLASH
                  $default
                  (select
                    (i32.const 0)
                    (select
                      (i32.const 1)
                      (i32.const 2)
                      (i32.eq (local.tee $char (i32.load8_u (local.get $offset))) (@char "\\")))
                    (i32.eq (local.get $char) (@char "\"")))))
              ;; If the character is a quote, terminate the string
              (return
                ;; FIXME: refactor byte copying into string method (making sure to update string capacity)
                (local.set $bytes_written
                  (i32.add
                    (local.get $bytes_written)
                    ;; Write the current chunk to the output string
                    (call $Json::copy_bytes
                      (local.get $chunk_offset)
                      (local.get $offset)
                      (i32.add (call $Term::String::get_offset (local.get $result)) (local.get $bytes_written)))))
                (call $Term::String::set::data::capacity
                  (local.get $result)
                  (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $bytes_written)) (i32.const 4)))
                (call $Term::String::init (local.get $result) (local.get $bytes_written))
                (i32.add (local.get $offset) (i32.const 1))))
            ;; If the character is a backslash, copy the chunk so far to the output string
            ;; FIXME: refactor byte copying into string method (making sure to update string capacity)
            (local.set $bytes_written
              (i32.add
                (local.get $bytes_written)
                ;; Write the current chunk to the output string
                (call $Json::copy_bytes
                  (local.get $chunk_offset)
                  (local.get $offset)
                  (i32.add (call $Term::String::get_offset (local.get $result)) (local.get $bytes_written)))))
            (call $Term::String::set::data::capacity
              (local.get $result)
              (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $bytes_written)) (i32.const 4)))
            ;; Write the decoded escape sequence to the output string
            (call $Json::write_escape_sequence
              ;; Skip over the backslash character
              (i32.add (local.get $offset) (i32.const 1))
              (call $Term::String::get_char_pointer (local.get $result) (local.get $bytes_written))
              (local.get $end_offset))
            (local.set $offset)
            (local.set $escape_sequence_length)
            (if
              ;; If the escape sequence was invalid, bail out
              (i32.eq (global.get $NULL) (local.get $escape_sequence_length))
              (then
                ;; Terminate the output string
                (call $Term::String::init (local.get $result) (local.get $bytes_written))
                (drop)
                ;; Return an error result
                (return (call $Json::parse_error (local.get $offset))))
              (else
                ;; Otherwise update the record of how many bytes have been written to the output string
                (local.set $bytes_written (i32.add (local.get $bytes_written) (local.get $escape_sequence_length)))
                ;; FIXME: string capacity update should be handled internally when writing the character
                (call $Term::String::set::data::capacity
                  (local.get $result)
                  (i32.div_u (call $Allocator::pad_to_4_byte_offset (local.get $bytes_written)) (i32.const 4)))
                (local.set $chunk_offset (local.get $offset))
                ;; Continue with the next character
                (br $LOOP))))
          ;; Otherwise if the character is not a special character, continue with the next character
          (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
          (br $LOOP)))))

  (func $Json::copy_bytes (param $start_offset i32) (param $end_offset i32) (param $target_offset i32) (result i32)
    (local $length i32)
    ;; Ensure enough space has been allocated
    (call $Allocator::extend
      (local.get $target_offset)
      (local.tee $length (i32.sub (local.get $end_offset) (local.get $start_offset))))
    ;; Copy the memory from the source address to the target address
    (memory.copy (local.get $target_offset) (local.get $start_offset) (local.get $length))
    ;; Return the number of bytes written
    (local.get $length))

  (func $Json::write_escape_sequence (param $source_offset i32) (param $target_offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    (if (result i32 i32)
      ;; If this is the end of the source string, return an error result
      (i32.eq (local.get $source_offset) (local.get $end_offset))
      (then
        (call $Json::parse_error (local.get $source_offset)))
      (else
        ;; Read the character at the current offset
        (local.set $char (i32.load8_u (local.get $source_offset)))
        ;; Determine the correct implementation depending on the escaped character
        (block $default
          (block $HEX
            (block $TAB
              (block $CARRIAGE_RETURN
                (block $NEW_LINE
                  (block $FORM_FEED
                    (block $BACKSPACE
                      (block $SLASH
                        (block $BACKSLASH
                          (block $DOUBLE_QUOTE
                            (br_table
                              $DOUBLE_QUOTE
                              $BACKSLASH
                              $SLASH
                              $BACKSPACE
                              $FORM_FEED
                              $NEW_LINE
                              $CARRIAGE_RETURN
                              $TAB
                              $HEX
                              $default
                              (select
                                (i32.const 0)
                                (select
                                  (i32.const 1)
                                  (select
                                    (i32.const 2)
                                    (select
                                      (i32.const 3)
                                      (select
                                        (i32.const 4)
                                        (select
                                          (i32.const 5)
                                          (select
                                            (i32.const 6)
                                            (select
                                              (i32.const 7)
                                              (select
                                                (i32.const 8)
                                                (i32.const 9)
                                                (i32.eq (@char "u") (local.get $char)))
                                              (i32.eq (@char "t") (local.get $char)))
                                            (i32.eq (@char "r") (local.get $char)))
                                          (i32.eq (@char "n") (local.get $char)))
                                        (i32.eq (@char "f") (local.get $char)))
                                      (i32.eq (@char "b") (local.get $char)))
                                    (i32.eq (@char "/") (local.get $char)))
                                  (i32.eq (@char "\\") (local.get $char)))
                                (i32.eq (@char "\"") (local.get $char)))))
                          (call $Json::write_char (@char "\"") (local.get $target_offset))
                          (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
                        (call $Json::write_char (@char "\\") (local.get $target_offset))
                        (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
                      (call $Json::write_char (@char "/") (local.get $target_offset))
                      (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
                    (call $Json::write_char (@char "\b") (local.get $target_offset))
                    (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
                  (call $Json::write_char (@char "\f") (local.get $target_offset))
                  (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
                (call $Json::write_char (@char "\n") (local.get $target_offset))
                (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
              (call $Json::write_char (@char "\r") (local.get $target_offset))
              (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
            (call $Json::write_char (@char "\t") (local.get $target_offset))
            (return (i32.const 1) (i32.add (local.get $source_offset) (i32.const 1))))
          (return
            (call $Json::parse_hex_escape_sequence
              (i32.add (local.get $source_offset) (i32.const 1))
              (local.get $target_offset)
              (local.get $end_offset))))
        ;; If no valid escape sequence was matched, return an error result
        (call $Json::parse_error (local.get $source_offset)))))

  (func $Json::write_char (param $char i32) (param $target_offset i32)
    ;; FIXME: refactor into string method (making sure to update string capacity)
    (call $Allocator::extend (local.get $target_offset) (i32.const 1))
    (i32.store8 (local.get $target_offset) (local.get $char)))

  (func $Json::parse_hex_escape_sequence (param $source_offset i32) (param $target_offset i32) (param $end_offset i32) (result i32 i32)
    (local $code_point i32)
    (call $Json::read_hex_code_point (local.get $source_offset) (local.get $end_offset))
    (local.set $source_offset)
    (local.set $code_point)
    (if (result i32 i32)
      (i32.eq (global.get $NULL) (local.get $code_point))
      (then
        (call $Json::parse_error (local.get $source_offset)))
      (else
        (block $default
          (block $quadruple
            (block $triple
              (block $double
                (block $single
                  (br_table
                    $single
                    $double
                    $triple
                    $quadruple
                    $default
                    (select
                      (i32.const 4)
                      (select
                        (i32.const 0)
                        (select
                          (i32.const 1)
                          (select
                            (i32.const 2)
                            (select
                              (i32.const 3)
                              (i32.const 4)
                              (call $Json::is_4_byte_code_point (local.get $code_point)))
                            (call $Json::is_3_byte_code_point (local.get $code_point)))
                          (call $Json::is_2_byte_code_point (local.get $code_point)))
                        (call $Json::is_1_byte_code_point (local.get $code_point)))
                      (i32.eq (global.get $NULL) (local.get $code_point)))))
                  (call $Json::write_1_byte_utf8_code_point (local.get $code_point) (local.get $target_offset))
                  (return (local.get $source_offset)))
                (call $Json::write_2_byte_utf8_code_point (local.get $code_point) (local.get $target_offset))
                (return (local.get $source_offset)))
              (call $Json::write_3_byte_utf8_code_point (local.get $code_point) (local.get $target_offset))
              (return (local.get $source_offset)))
            (call $Json::write_4_byte_utf8_code_point (local.get $code_point) (local.get $target_offset))
            (return (local.get $source_offset)))
        ;; Default implementation
        (call $Json::parse_error (local.get $source_offset)))))

  (func $Json::is_1_byte_code_point (param $code_point i32) (result i32)
    (i32.le_u (local.get $code_point) (i32.const 0x7F)))

  (func $Json::is_2_byte_code_point (param $code_point i32) (result i32)
    (i32.le_u (local.get $code_point) (i32.const 0x7FF)))

  (func $Json::is_3_byte_code_point (param $code_point i32) (result i32)
    (i32.le_u (local.get $code_point) (i32.const 0xFFFF)))

  (func $Json::is_4_byte_code_point (param $code_point i32) (result i32)
    (i32.le_u (local.get $code_point) (i32.const 0x10FFFF)))

  (func $Json::read_hex_code_point (param $source_offset i32) (param $end_offset i32) (result i32 i32)
    (local $code_point i32)
    (local $low_surrogate i32)
    ;; Read the hex-escaped code point
    (call $Json::read_hex_word (local.get $source_offset) (local.get $end_offset))
    (local.set $source_offset)
    (if (result i32 i32)
      ;; Determine whether the escape sequence encodes a UTF-16 surrogate pair
      (call $Json::is_utf_surrogate_pair (local.tee $code_point))
      (then
        (if (result i32 i32)
          (i32.and
            ;; Confirm that the first half of the surrogate pair is the high surrogate
            (call $Json::is_utf_high_surrogate (local.get $code_point))
            ;; Confirm that the high surrogate is immediately followed by another hex-escaped code point (the low surrogate)
            (if (result i32)
              (i32.lt_u (local.get $end_offset) (i32.add (local.get $source_offset) (i32.const 2)))
              (then
                (i32.const 0))
              (else
                (i32.eq
                  (i32.load16_u (local.get $source_offset))
                  (i32.or (@char "\\") (i32.shl (@char "u") (i32.const 8)))))))
          (then
            ;; Read the next hex-encoded value immediately following the escape sequence prefix
            (call $Json::read_hex_word (i32.add (local.get $source_offset) (i32.const 2)) (local.get $end_offset))
            (local.set $source_offset)
            (if (result i32 i32)
              ;; Confirm that the second half of the surrogate pair is the low surrogate
              (call $Json::is_utf_low_surrogate (local.tee $low_surrogate))
              (then
                ;; Calculate the combined code point
                (call $Json::get_utf_surrogate_pair_code_point (local.get $code_point) (local.get $low_surrogate))
                (local.get $source_offset))
              (else
                (call $Json::parse_error (local.get $source_offset)))))
          (else
            (call $Json::parse_error (local.get $source_offset)))))
      (else
        (local.get $code_point)
        (local.get $source_offset))))

  (func $Json::is_utf_surrogate_pair (param $code_point i32) (result i32)
    (i32.or
      (call $Json::is_utf_high_surrogate (local.get $code_point))
      (call $Json::is_utf_low_surrogate (local.get $code_point))))

  (func $Json::is_utf_high_surrogate (param $code_point i32) (result i32)
    (i32.and
      (i32.ge_u (local.get $code_point) (i32.const 0xD800))
      (i32.le_u (local.get $code_point) (i32.const 0xDBFF))))

  (func $Json::is_utf_low_surrogate (param $code_point i32) (result i32)
    (i32.and
      (i32.ge_u (local.get $code_point) (i32.const 0xDC00))
      (i32.le_u (local.get $code_point) (i32.const 0xDFFF))))

  (func $Json::get_utf_surrogate_pair_code_point (param $high_surrogate i32) (param $low_surrogate i32) (result i32)
    (i32.add
      (i32.mul (i32.sub (local.get $high_surrogate) (i32.const 0xD800)) (i32.const 0x400))
      (i32.add (i32.sub (local.get $low_surrogate) (i32.const 0xDC00)) (i32.const 0x10000))))

  (func $Json::read_hex_word (param $source_offset i32) (param $end_offset i32) (result i32 i32)
    (local $digits i32)
    (local $digit1 i32)
    (local $digit2 i32)
    (local $digit3 i32)
    (local $digit4 i32)
    (if (result i32 i32)
      (i32.lt_u (local.get $end_offset) (i32.add (local.get $source_offset) (i32.const 4)))
      (then
        (call $Json::parse_error (local.get $source_offset)))
      (else
        (local.set $digits (i32.load (local.get $source_offset)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.tee $digit1 (call $Json::parse_hex_digit (call $Utils::i32::get_byte (local.get $digits) (i32.const 0)))))
            (i32.or
              (i32.eq (global.get $NULL) (local.tee $digit2 (call $Json::parse_hex_digit (call $Utils::i32::get_byte (local.get $digits) (i32.const 1)))))
              (i32.or
                (i32.eq (global.get $NULL) (local.tee $digit3 (call $Json::parse_hex_digit (call $Utils::i32::get_byte (local.get $digits) (i32.const 2)))))
                (i32.eq (global.get $NULL) (local.tee $digit4 (call $Json::parse_hex_digit (call $Utils::i32::get_byte (local.get $digits) (i32.const 3))))))))
          (then
            (call $Json::parse_error (local.get $source_offset)))
          (else
            (i32.or
              (i32.shl (local.get $digit1) (i32.const 12))
              (i32.or
                (i32.shl (local.get $digit2) (i32.const 8))
                (i32.or
                  (i32.shl (local.get $digit3) (i32.const 4))
                  (local.get $digit4))))
            (i32.add (local.get $source_offset) (i32.const 4)))))))

  (func $Json::parse_hex_digit (param $char i32) (result i32)
    (select
      (i32.sub (local.get $char) (@char "0"))
      (select
        (i32.add (i32.const 0x0A) (i32.sub (local.get $char) (@char "A")))
        (select
          (i32.add (i32.const 0x0A) (i32.sub (local.get $char) (@char "a")))
          (global.get $NULL)
          (i32.and
            (i32.ge_u (local.get $char) (@char "a"))
            (i32.le_u (local.get $char) (@char "f"))))
        (i32.and
          (i32.ge_u (local.get $char) (@char "A"))
          (i32.le_u (local.get $char) (@char "F"))))
      (i32.and
        (i32.ge_u (local.get $char) (@char "0"))
        (i32.le_u (local.get $char) (@char "9")))))

  (func $Json::write_1_byte_utf8_code_point (param $code i32) (param $target_offset i32) (result i32)
    (call $Allocator::extend (local.get $target_offset) (i32.const 1))
    (i32.store8 (local.get $target_offset) (local.get $code))
    (i32.const 1))

  (func $Json::write_2_byte_utf8_code_point (param $code i32) (param $target_offset i32) (result i32)
    (call $Allocator::extend (local.get $target_offset) (i32.const 2))
    (i32.store16
      (local.get $target_offset)
      ;; The UTF-8 bytes are laid out in reverse order due to little-endian linear memory integer representation
      (i32.or
        (i32.or
          (i32.const 0xC0)
          (i32.shr_u (local.get $code) (i32.const 6)))
        (i32.shl
          (i32.or
            (i32.const 0x80)
            (i32.and (i32.const 0x3F) (local.get $code)))
          (i32.const 8))))
    (i32.const 2))

  (func $Json::write_3_byte_utf8_code_point (param $code i32) (param $target_offset i32) (result i32)
    (call $Allocator::extend (local.get $target_offset) (i32.const 3))
    (i32.store16
      (local.get $target_offset)
      ;; The UTF-8 bytes are laid out in reverse order due to little-endian linear memory integer representation
      (i32.or
        (i32.or
          (i32.const 0xE0)
          (i32.shr_u (local.get $code) (i32.const 12)))
        (i32.shl
          (i32.or
            (i32.const 0x80)
            (i32.and (i32.const 0x3F) (i32.shr_u (local.get $code) (i32.const 6))))
          (i32.const 8))))
    (i32.store8
      (i32.add (local.get $target_offset) (i32.const 2))
      (i32.or
        (i32.const 0x80)
        (i32.and (i32.const 0x3F) (local.get $code))))
    (i32.const 3))

  (func $Json::write_4_byte_utf8_code_point (param $code i32) (param $target_offset i32) (result i32)
    (call $Allocator::extend (local.get $target_offset) (i32.const 4))
    (i32.store
      (local.get $target_offset)
      ;; The UTF-8 bytes are laid out in reverse order due to little-endian linear memory integer representation
      (i32.or
        (i32.or
          (i32.or
            (i32.const 0xF0)
            (i32.shr_u (local.get $code) (i32.const 18)))
          (i32.shl
            (i32.or
              (i32.const 0x80)
              (i32.and (i32.const 0x3F) (i32.shr_u (local.get $code) (i32.const 12))))
            (i32.const 8)))
        (i32.or
          (i32.shl
            (i32.or
              (i32.const 0x80)
              (i32.and (i32.const 0x3F) (i32.shr_u (local.get $code) (i32.const 6))))
            (i32.const 16))
          (i32.shl
            (i32.or
              (i32.const 0x80)
              (i32.and (i32.const 0x3F) (local.get $code)))
            (i32.const 24)))))
    (i32.const 4))

  (func $Json::parse_array (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    (local $results i32)
    (local $value i32)
    ;; Skip any whitespace that follows the initial opening brace
    (call $Json::skip_whitespace (i32.add (local.get $offset) (i32.const 1)) (local.get $end_offset))
    (local.set $char)
    (local.set $offset)
    (if (result i32 i32)
      ;; If an empty JSON array was provided, return the empty list
      (i32.eq (local.get $char) (@char "]"))
      (then
        (call $Term::List::empty)
        (i32.add (local.get $offset) (i32.const 1)))
      (else
        ;; Otherwise allocate a new list to hold the results
        (local.set $results (call $Term::List::allocate_unsized))
        ;; Iterate through each of the JSON values in the array
        (loop $LOOP (result i32 i32)
          ;; Parse the next array item
          (call $Json::parse_value (local.get $offset) (local.get $end_offset))
          (local.set $offset)
          (local.set $value)
          (if (result i32 i32)
            ;; If an invalid JSON value was encountered, terminate the results list and return an error result
            (i32.eq (global.get $NULL) (local.get $value))
            (then
              (call $Term::List::init_unsized (local.get $results))
              (drop)
              (call $Json::parse_error (local.get $offset)))
            (else
              ;; Otherwise add the parsed item to the results list
              (local.set $results (call $Term::List::append_unsized (local.get $results) (local.get $value)))
              ;; Skip any trailing whitespace and peek the next lookahead character
              (call $Json::skip_whitespace (local.get $offset) (local.get $end_offset))
              (local.set $char)
              (local.set $offset)
              ;; Determine the behavior based on the lookahead character
              (block $default
                (block $COMMA
                  (block $CLOSE_BRACE
                    (br_table
                      $CLOSE_BRACE
                      $COMMA
                      $default
                      (select
                        (i32.const 0)
                        (select
                          (i32.const 1)
                          (i32.const 2)
                          (i32.eq (@char ",") (local.get $char)))
                        (i32.eq (@char "]") (local.get $char)))))
                  ;; If the lookahead character is the closing brace character, return the initialized list
                  (return
                    (call $Term::List::init_unsized (local.get $results))
                    (i32.add (local.get $offset) (i32.const 1))))
                ;; If the lookahead character is a comma, continue with the next item
                (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
                (br $LOOP))
              ;; Otherwise terminate the list and return an error result
              (call $Term::List::init_unsized (local.get $results))
              (drop)
              (call $Json::parse_error (local.get $offset))))))))

  (func $Json::parse_object (param $offset i32) (param $end_offset i32) (result i32 i32)
    (local $char i32)
    (local $keys i32)
    (local $values i32)
    (local $value i32)
    ;; Skip any whitespace that follows the initial opening brace
    (call $Json::skip_whitespace (i32.add (local.get $offset) (i32.const 1)) (local.get $end_offset))
    (local.set $char)
    (local.set $offset)
    (if (result i32 i32)
      ;; If an empty JSON object was provided, return the empty record
      (i32.eq (local.get $char) (@char "}"))
      (then
        (call $Term::Record::empty)
        (i32.add (local.get $offset) (i32.const 1)))
      (else
        ;; Otherwise allocate a new key and value lists to hold the results
        (local.set $keys (call $Term::List::allocate_unsized))
        (local.set $values (call $Term::List::allocate_unsized))
        ;; Iterate through each of the JSON fields in the object
        (loop $LOOP (result i32 i32)
          ;; Parse the field key
          (call $Json::parse_value (local.get $offset) (local.get $end_offset))
          (local.set $offset)
          (local.set $value)
          (if (result i32 i32)
            ;; If an invalid JSON key was encountered, terminate the key and value lists and return an error result
            (i32.or
              (i32.eq (global.get $NULL) (local.get $value))
              (i32.eqz (call $Term::String::is (local.get $value))))
            (then
              (call $Term::List::init_unsized (local.get $keys))
              (drop)
              (call $Term::List::init_unsized (local.get $values))
              (drop)
              (call $Json::parse_error (local.get $offset)))
            (else
              ;; Otherwise add the parsed key to the keys list
              (local.set $keys (call $Term::List::append_unsized (local.get $keys) (local.get $value)))
              ;; Skip any trailing whitespace and peek the next lookahead character
              (call $Json::skip_whitespace (local.get $offset) (local.get $end_offset))
              (local.set $char)
              (local.set $offset)
              (if (result i32 i32)
                ;; If the key is not followed by a colon separator, terminate the key and value lists and return an error result
                (i32.ne (local.get $char) (@char ":"))
                (then
                  (call $Term::List::init_unsized (local.get $keys))
                  (drop)
                  (call $Term::List::init_unsized (local.get $values))
                  (drop)
                  (call $Json::parse_error (local.get $offset)))
                (else
                  ;; Parse the field value, skipping over the colon separator
                  (call $Json::parse_value (i32.add (local.get $offset) (i32.const 1)) (local.get $end_offset))
                  (local.set $offset)
                  (local.set $value)
                  (if (result i32 i32)
                    ;; If an invalid JSON value was encountered, terminate the key and value lists and return an error result
                    (i32.eq (global.get $NULL) (local.get $value))
                    (then
                      (call $Term::List::init_unsized (local.get $keys))
                      (drop)
                      (call $Term::List::init_unsized (local.get $values))
                      (drop)
                      (call $Json::parse_error (local.get $offset)))
                    (else
                      ;; Otherwise add the parsed value to the values list
                      (local.set $values (call $Term::List::append_unsized (local.get $values) (local.get $value)))
                      ;; Skip any trailing whitespace and peek the next lookahead character
                      (call $Json::skip_whitespace (local.get $offset) (local.get $end_offset))
                      (local.set $char)
                      (local.set $offset)
                      ;; Determine the behavior based on the lookahead character
                      (block $default
                        (block $COMMA
                          (block $CLOSE_CURLY
                            (br_table
                              $CLOSE_CURLY
                              $COMMA
                              $default
                              (select
                                (i32.const 0)
                                (select
                                  (i32.const 1)
                                  (i32.const 2)
                                  (i32.eq (@char ",") (local.get $char)))
                                (i32.eq (@char "}") (local.get $char)))))
                          ;; If the lookahead character is the closing brace character, return the initialized record
                          (return
                            (call $Term::Record::new
                              (call $Term::List::init_unsized (local.get $keys))
                              (call $Term::List::init_unsized (local.get $values)))
                            (i32.add (local.get $offset) (i32.const 1))))
                        ;; If the lookahead character is a comma, continue with the next field
                        (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
                        (br $LOOP))
                      ;; Otherwise terminate the key and value lists and return an error result
                      (call $Term::List::init_unsized (local.get $keys))
                      (drop)
                      (call $Term::List::init_unsized (local.get $values))
                      (drop)
                      (call $Json::parse_error (local.get $offset)))))))))))))
