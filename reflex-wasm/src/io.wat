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
      (call $Term::Cell::get_field_pointer (global.get $Io::IOVEC) (i32.const 2)))))
