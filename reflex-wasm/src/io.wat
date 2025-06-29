;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Constants for the stdio file descriptors
  (global $Io::STDIN i32 (i32.const 0))
  (global $Io::STDOUT i32 (i32.const 1))
  (global $Io::STDERR i32 (i32.const 2))

  ;; Initialize the private use area that holds IO intermediate value scatter/gather vectors
  (@const $Io::IOVEC i32 (call $Term::Cell::allocate (i32.const 3)))

  (func $Io::write_stdout (param $offset i32) (param $length i32) (result i32)
    (call $Io::write_bytes (global.get $Io::STDOUT) (local.get $offset) (local.get $length)))

  (func $Io::write_stderr (param $offset i32) (param $length i32) (result i32)
    (call $Io::write_bytes (global.get $Io::STDERR) (local.get $offset) (local.get $length)))

  (func $Io::write_bytes (param $fd i32) (param $offset i32) (param $length i32) (result i32)
    ;; Store the pointer and length in the IO private use area
    (call $Term::Cell::set_field (global.get $Io::IOVEC) (i32.const 0) (local.get $offset))
    (call $Term::Cell::set_field (global.get $Io::IOVEC) (i32.const 1) (local.get $length))
    ;; Write to WASI stdout
    (call $WASI::fd_write
      ;; File descriptor ID
      (local.get $fd)
      ;;; Pointer to the array of IO vectors from which to retrieve data
      (call $Term::Cell::get_field_pointer (global.get $Io::IOVEC) (i32.const 0))
      ;; Number of IO vector entries to read
      (i32.const 1)
      ;;; The memory address at which to write the number of bytes written
      (call $Term::Cell::get_field_pointer (global.get $Io::IOVEC) (i32.const 2))))

  (func $Io::log_term (param $value i32) (result i32)
    (local $serialized_value i32)
    (local $bytes_written i32)
    ;; Serialize the term to a temporary string term
    (local.set $serialized_value (call $Term::String::from (local.get $value)))
    ;; Write the serialized string contents to stdout
    (call $Io::write_stdout
      (call $Term::String::get_offset (local.get $serialized_value))
      (call $Term::String::get_length (local.get $serialized_value)))
    ;; Store the number of bytes written
    (local.set $bytes_written)
    ;; Dispose of the temporary string term
    (if
      (i32.ne (local.get $serialized_value) (local.get $value))
      (then
        (call $Term::String::drop (local.get $serialized_value))))
    ;; Write a newline to stdout
    (call $Io::write_stdout
      (call $Term::String::get_offset (global.get $Stdlib_Log::NEWLINE))
      (call $Term::String::get_length (global.get $Stdlib_Log::NEWLINE)))
    ;; Return the total number of bytes written
    (i32.add (local.get $bytes_written))))
