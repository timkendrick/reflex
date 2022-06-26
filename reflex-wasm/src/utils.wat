;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Utils::i32::abs (param $self i32) (result i32)
    (i32.xor
      (i32.add (local.get $self) (local.tee $self (i32.shr_s (local.get $self) (i32.const 31))))
      (local.get $self)))

  (func $Utils::i32::min_s (param $self i32) (param $other i32) (result i32)
    (select
      (local.get $self)
      (local.get $other)
      (i32.lt_s (local.get $self) (local.get $other))))

  (func $Utils::i32::max_s (param $self i32) (param $other i32) (result i32)
    (select
      (local.get $self)
      (local.get $other)
      (i32.gt_s (local.get $self) (local.get $other))))

  (func $Utils::i32::min_u (param $self i32) (param $other i32) (result i32)
    (select
      (local.get $self)
      (local.get $other)
      (i32.lt_u (local.get $self) (local.get $other))))

  (func $Utils::i32::max_u (param $self i32) (param $other i32) (result i32)
    (select
      (local.get $self)
      (local.get $other)
      (i32.gt_u (local.get $self) (local.get $other))))

  (func $Utils::i32::saturating_sub_u (param $self i32) (param $other i32) (result i32)
    (select
      (i32.sub (local.get $self) (local.get $other))
      (i32.const 0)
      (i32.le_u (local.get $other) (local.get $self))))

  (func $Utils::i32::pow (param $self i32) (param $exponent i32) (result i32)
    ;; See https://rosettacode.org/wiki/Exponentiation_operator#C
    (local $result i32)
    (local.set $result (i32.const 1))
    (loop $LOOP
      (local.set $result
        (select
          (i32.mul (local.get $result) (local.get $self))
          (local.get $result)
          (i32.and (local.get $exponent) (i32.const 0x00000001))))
      (local.set $self (i32.mul (local.get $self) (local.get $self)))
      (br_if $LOOP (local.tee $exponent (i32.shr_u (local.get $exponent) (i32.const 1)))))
    (local.get $result))

  (func $Utils::i32::neg (param $self i32) (result i32)
    (i32.sub (i32.const 0) (local.get $self)))

  (func $Utils::i32::get_byte (param $self i32) (param $index i32) (result i32)
    (i32.and (i32.const 0xFF) (i32.shr_u (local.get $self) (i32.mul (local.get $index) (i32.const 8)))))

  (func $Utils::i32::round_to_next (param $self i32) (param $step i32) (result i32)
    ;; (step * ((self + (step - 1)) / step))
    (i32.mul (i32.div_u (i32.add (local.get $self) (i32.sub (local.get $step) (i32.const 1))) (local.get $step)) (local.get $step)))

  (func $Utils::i64::get_byte (param $self i64) (param $index i32) (result i32)
    (i32.and (i32.const 0xFF) (i32.wrap_i64 (i64.shr_u (local.get $self) (i64.extend_i32_u (i32.mul (local.get $index) (i32.const 8)))))))

  (func $Utils::f64::is_integer (param $self f64) (result i32)
    (f64.eq (local.get $self) (f64.convert_i32_s (i32.trunc_f64_s (local.get $self)))))

  (func $Utils::f64::is_nan (param $self f64) (result i32)
    (f64.ne (local.get $self) (local.get $self)))

  (func $Utils::f64::is_infinite (param $self f64) (result i32)
    (i32.or (f64.eq (local.get $self) (f64.const inf)) (f64.eq (local.get $self) (f64.const -inf))))

  (func $Utils::f64::is_finite (param $self f64) (result i32)
    (i32.and
      (i32.eqz (call $Utils::f64::is_nan (local.get $self)))
      (i32.eqz (call $Utils::f64::is_infinite (local.get $self)))))

  (func $Utils::f64::neg (param $self f64) (result f64)
    (f64.sub (f64.const 0) (local.get $self)))

  (func $Utils::f64::remainder_int (param $self f64) (param $divisor i32) (result f64)
    (local $integer_value i32)
    (f64.add
      (f64.convert_i32_s
        (i32.rem_s
          (local.tee $integer_value (i32.trunc_f64_s (local.get $self)))
          (local.get $divisor)))
      (f64.sub
        (local.get $self)
        (f64.convert_i32_s (local.get $integer_value)))))

  (func $Utils::f64::pow_int (param $self f64) (param $exponent i32) (result f64)
    ;; See https://rosettacode.org/wiki/Exponentiation_operator#C
    (local $result f64)
    (local.set $result (f64.const 1))
    (if (result f64 i32)
      (i32.lt_s (local.get $exponent) (i32.const 0))
      (then
        (f64.div (f64.const 1.0) (local.get $self))
        (i32.sub (i32.const 0) (local.get $exponent)))
      (else
        (local.get $self)
        (local.get $exponent)))
    (local.set $exponent)
    (local.set $self)
    (loop $LOOP
      (local.set $result
        (select
          (f64.mul (local.get $result) (local.get $self))
          (local.get $result)
          (i32.and (local.get $exponent) (i32.const 0x00000001))))
      (local.set $self (f64.mul (local.get $self) (local.get $self)))
      (br_if $LOOP (local.tee $exponent (i32.shr_u (local.get $exponent) (i32.const 1)))))
    (local.get $result))

  (func $Utils::u8_array::equals (param $left_offset i32) (param $left_length i32) (param $right_offset i32) (param $right_length i32) (result i32)
    (local $index i32)
    (if (result i32)
      (i32.ne (local.get $left_length) (local.get $right_length))
      (then
        (global.get $FALSE))
      (else
        (if (result i32)
          (i32.eq (local.get $left_length) (i32.const 0))
          (then
            (global.get $TRUE))
          (else
            (loop $LOOP (result i32)
              (if (result i32)
                (i32.ne
                  (i32.load8_u (i32.add (local.get $left_offset) (local.get $index)))
                  (i32.load8_u (i32.add (local.get $right_offset) (local.get $index))))
                (then
                  (global.get $FALSE))
                (else
                  (if (result i32)
                    (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $left_length))
                    (then
                      (global.get $TRUE))
                    (else
                      (br $LOOP)))))))))))

  (func $Utils::i32_array::equals (param $left_offset i32) (param $left_length i32) (param $right_offset i32) (param $right_length i32) (result i32)
    (local $index i32)
    (if (result i32)
      (i32.ne (local.get $left_length) (local.get $right_length))
      (then
        (global.get $FALSE))
      (else
        (if (result i32)
          (i32.eq (local.get $left_length) (i32.const 0))
          (then
            (global.get $TRUE))
          (else
            (loop $LOOP (result i32)
              (if (result i32)
                (i32.ne
                  (i32.load (i32.add (local.get $left_offset) (i32.mul (local.get $index) (i32.const 4))))
                  (i32.load (i32.add (local.get $right_offset) (i32.mul (local.get $index) (i32.const 4)))))
                (then
                  (global.get $FALSE))
                (else
                  (if (result i32)
                    (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $left_length))
                    (then
                      (global.get $TRUE))
                    (else
                      (br $LOOP)))))))))))

  (func $Utils::bytes::equals (param $left_offset i32) (param $left_length i32) (param $right_offset i32) (param $right_length i32) (result i32)
    ;; First compare as much as possible of the input arrays as 4-byte i32 arrays
    (local $left_i32_length i32)
    (local $right_i32_length i32)
    (local.set $left_i32_length (i32.div_u (local.get $left_length) (i32.const 4)))
    (local.set $right_i32_length (i32.div_u (local.get $right_length) (i32.const 4)))
    (if (result i32)
      (call $Utils::i32_array::equals (local.get $left_offset) (local.get $left_i32_length) (local.get $right_offset) (local.get $right_i32_length))
      (then
        ;; Then compare any remaining bytes left over after the i32 array comparison
        (local.set $left_i32_length (i32.mul (local.get $left_i32_length) (i32.const 4)))
        (local.set $right_i32_length (i32.mul (local.get $right_i32_length) (i32.const 4)))
        (call $Utils::u8_array::equals
          (i32.add (local.get $left_offset) (local.get $left_i32_length)) (i32.sub (local.get $left_length) (local.get $left_i32_length))
          (i32.add (local.get $right_offset) (local.get $right_i32_length)) (i32.sub (local.get $right_length) (local.get $right_i32_length))))
      (else
        (global.get $FALSE))))

  (func $Utils::bytes::write_json (param $source_offset i32) (param $source_length i32) (param $target_offset i32) (result i32)
    (local $original_offset i32)
    (local $index i32)
    (local $is_special_char i32)
    (if (result i32)
      ;; If the input string is empty, write an empty JSON string literal
      (i32.eqz (local.get $source_length))
      (then
        ;; Allocate two bytes for opening and closing quotes and write the characters to the output
        (call $Allocator::extend (local.get $target_offset) (i32.const 2))
        (i32.store8 offset=0 (local.get $target_offset) (@char "\""))
        (i32.store8 offset=1 (local.get $target_offset) (@char "\""))
        ;; Return the number of bytes written
        (i32.const 2))
      (else
        ;; Keep track of the original offset to work out the number of bytes written
        (local.set $original_offset (local.get $target_offset))
        ;; Allocate a single byte for the opening quote, write the byte and update the current offset accordingly
        (call $Allocator::extend (local.get $target_offset) (i32.const 1))
        (i32.store8 (local.get $target_offset) (@char "\""))
        (local.set $target_offset (i32.add (local.get $target_offset) (i32.const 1)))
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
                  (local.tee $is_special_char
                    (call $Utils::u8::is_json_special_char
                      (i32.load8_u (i32.add (local.get $source_offset) (local.get $index))))))
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $source_length)))))
          ;; If this is a special character, decrement the index to counteract the superfluous addition within the loop
          (local.set $index (i32.sub (local.get $index) (local.get $is_special_char)))
          ;; Allocate enough space to store the chunk, plus the escape sequence if applicable
          (call $Allocator::extend
            (local.get $target_offset)
            (i32.add
              (local.get $index)
              ;; Special characters need two bytes to store the backslash-escaped sequence
              (i32.add (local.get $is_special_char) (local.get $is_special_char))))
          ;; Copy the chunk into the output string
          (memory.copy
            (local.get $target_offset)
            (local.get $source_offset)
            (local.get $index))
          ;; Update the current offset to reflect the addition of the chunk
          (local.set $target_offset (i32.add (local.get $target_offset) (local.get $index)))
          (if
            ;; If we stopped at a special character, write its escaped form into the output
            (local.get $is_special_char)
            (then
              ;; Write a backslash character to the output
              (i32.store8 offset=0 (local.get $target_offset) (@char "\\"))
              ;; Retrieve the correct escape code and write it to the output
              (i32.store8 offset=1
                (local.get $target_offset)
                (call $Utils::u8::get_json_escape_code
                  (i32.load8_u (i32.add (local.get $source_offset) (local.get $index)))))
              ;; Update the target offset to reflect the addition of the escape sequence
              (local.set $target_offset (i32.add (local.get $target_offset) (i32.const 2)))
              ;; Update the source offset and remaining length, skipping over the escaped character
              (local.set $source_offset (i32.add (local.get $source_offset) (local.tee $index (i32.add (local.get $index) (i32.const 1)))))
              (local.tee $source_length (i32.sub (local.get $source_length) (local.get $index)))
              (local.set $index (i32.const 0))
              ;; If the remaining length is non-zero, continue with the next chunk
              (br_if $LOOP))
            (else)))
          ;; Allocate a final byte for the closing quote, write the quote, and return the final number of bytes written
          (call $Allocator::extend (local.get $target_offset) (i32.const 1))
          (i32.store8 (local.get $target_offset) (@char "\""))
          (i32.sub (i32.add (local.get $target_offset) (i32.const 1)) (local.get $original_offset)))))

  (func $Utils::u8::is_json_special_char (param $char i32) (result i32)
    (i32.or
      (i32.eq (local.get $char) (@char "\\"))
      (i32.or
        (i32.eq (local.get $char) (@char "\""))
        (i32.or
          (i32.eq (local.get $char) (@char "\t"))
          (i32.or
            (i32.eq (local.get $char) (@char "\r"))
            (i32.or
              (i32.eq (local.get $char) (@char "\n"))
              (i32.or
                (i32.eq (local.get $char) (@char "\f"))
                (i32.eq (local.get $char) (@char "\b")))))))))

  (func $Utils::u8::get_json_escape_code (param $char i32) (result i32)
    ;; Given an ASCII byte, return the corresponding character to use as the second character in a
    ;; backslash-escaped JSON escape sequence, or zero if this character does not need escaping
    (@switch
      (@list
        (@list
          (i32.eq (local.get $char) (@char "\b"))
          (return (@char "b")))
        (@list
          (i32.eq (local.get $char) (@char "\f"))
          (return (@char "f")))
        (@list
          (i32.eq (local.get $char) (@char "\n"))
          (return (@char "n")))
        (@list
          (i32.eq (local.get $char) (@char "\r"))
          (return (@char "r")))
        (@list
          (i32.eq (local.get $char) (@char "\t"))
          (return (@char "t")))
        (@list
          (i32.eq (local.get $char) (@char "\""))
          (return (@char "\"")))
        (@list
          (i32.eq (local.get $char) (@char "\\"))
          (return (@char "\\"))))
      ;; Default case
      (i32.const 0)))

  (func $Utils::u8::write_decimal_digit (param $digit i32) (param $offset i32)
    (i32.store8 (local.get $offset) (i32.add (@char "0") (local.get $digit))))

  (func $Utils::i32::write_string (param $value i32) (param $offset i32) (result i32)
    (local $is_negative i32)
    (local $remaining_digits i32)
    (local $num_chars i32)
    (local $index i32)
    ;; Find out how many bytes to allocate for the string representation
    ;; Assign a temporary value to determine the number of digits of the positive integer
    (local.set $remaining_digits (call $Utils::i32::abs (local.get $value)))
    ;; Keep track of whether the number is negative
    (local.set $is_negative (i32.ne (local.get $remaining_digits) (local.get $value)))
    ;; Update the value the absolute value for use later
    (local.set $value (local.get $remaining_digits))
    (loop $LOOP
      ;; Increment the length
      (local.set $num_chars (i32.add (local.get $num_chars) (i32.const 1)))
      ;; If the temporary value is still greater than zero after being divided by 10, continue with the next digit
      (br_if $LOOP (local.tee $remaining_digits (i32.div_u (local.get $remaining_digits) (i32.const 10)))))
    ;; Allocate the required number of bytes to store the string representation,
    ;; allowing an extra byte for the minus sign if the number is negative
    (call $Allocator::extend (local.get $offset) (local.tee $num_chars (i32.add (local.get $num_chars) (local.get $is_negative))))
    ;; Reset the temporary value to the absolute value
    (local.set $remaining_digits (local.get $value))
    ;; If the number is negative, add the minus sign
    (if
      (local.get $is_negative)
      (then
        (i32.store8 (local.get $offset) (@char "-")))
      (else))
    ;; Push the length onto the stack as the return value
    (local.get $num_chars)
    ;; Write the bytes in reverse order, starting from the least significant digit
    (loop $LOOP
      ;; Write the current least significant digit to the output and increment the offset
      (call $Utils::u8::write_decimal_digit
        (i32.rem_u (local.get $remaining_digits) (i32.const 10))
        (i32.add (local.get $offset) (local.tee $num_chars (i32.sub (local.get $num_chars) (i32.const 1)))))
      ;; If the temporary value is still greater than zero after being divided by 10, continue with the next digit
      (br_if $LOOP (local.tee $remaining_digits (i32.div_u (local.get $remaining_digits) (i32.const 10))))))

  (func $Utils::f64::write_string (param $value f64) (param $offset i32) (result i32)
    (local $original_offset i32)
    (local $integers f64)
    (local $multiplier f64)
    (local $num_decimals i32)
    (local $remaining_decimals f64)
    (local $next_decimal f64)
    (if (result i32)
      ;; If the value is NaN or infinite, return the null sentinel value
      (i32.eqz (call $Utils::f64::is_finite (local.get $value)))
      (then
        (global.get $NULL))
      (else
        ;; Keep track of the original offset to determine how many bytes were written
        (local.set $original_offset (local.get $offset))
        ;; Determine how many digits are needed to represent the decimal portion of the number
        (local.set $multiplier (f64.const 1))
        (loop $LOOP
          ;; Increment the number of decimals
          (local.set $num_decimals (i32.add (local.get $num_decimals) (i32.const 1)))
          ;; Loop until round(value * multiplier) / multiplier == value
          (br_if $LOOP
            (i32.and
              (f64.ne
                (f64.div
                  (f64.nearest
                    (f64.mul
                      (local.get $value)
                      ;; Increase the multiplier by a factor of 10 on each iteration
                      (local.tee $multiplier (f64.mul (local.get $multiplier) (f64.const 10)))))
                  (local.get $multiplier))
                (local.get $value))
              ;; Enforce a maximum number of decimal places to avoid potential infinite loops
              (i32.lt_u (local.get $num_decimals) (i32.const 32)))))
        ;; Write the integer portion of the number
        (call $Utils::i32::write_string
          (i32.trunc_f64_s (local.tee $integers (f64.trunc (local.get $value))))
          (local.get $offset))
        ;; Allocate the required number of bytes to store the decimal point character followed by the decimal digits
        (call $Allocator::extend
          (local.tee $offset (i32.add (local.get $offset)))
          (i32.add (i32.const 1) (local.get $num_decimals)))
        ;; Write the decimal point character and increment the current offset
        (i32.store8 (local.get $offset) (@char "."))
        (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
        ;; Write the decimal digits
        (local.set $remaining_decimals (f64.abs (f64.sub (local.get $value) (local.get $integers))))
        (if
          (i32.gt_u (local.get $num_decimals) (i32.const 1))
          (then
            ;; Write all but the final decimals by multiplying by 10 and truncating
            (loop $LOOP
              (call $Utils::u8::write_decimal_digit
                (i32.trunc_f64_u
                  (local.tee $next_decimal
                    (f64.trunc (local.tee $remaining_decimals (f64.mul (local.get $remaining_decimals) (f64.const 10))))))
                (local.get $offset))
              (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
              (local.set $remaining_decimals (f64.sub (local.get $remaining_decimals) (local.get $next_decimal)))
              (br_if $LOOP (i32.gt_u (local.tee $num_decimals (i32.sub (local.get $num_decimals) (i32.const 1))) (i32.const 1)))))
          (else))
        ;; Write the final decimal by mutiplying by 10 and rounding
        (call $Utils::u8::write_decimal_digit
          (i32.trunc_f64_u (f64.nearest (f64.mul (local.get $remaining_decimals) (f64.const 10))))
          (local.get $offset))
        ;; Increment the current offset to reflect the final digit
        (i32.add (local.get $offset) (i32.const 1))
        ;; Subtract the original offset to calculate the number of bytes written
        (i32.sub (local.get $original_offset))))))
