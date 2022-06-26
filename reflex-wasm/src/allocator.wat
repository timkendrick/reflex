;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (export "memory" (memory $Allocator::heap))
  ;; Initialize linear memory for the heap with an initial page size of 1
  ;; (additional pages will be allocated during runtime if necessary)
  ;; https://webassembly.github.io/spec/core/exec/runtime.html#page-size
  (memory $Allocator::heap 1)
  (global $Allocator::heap_capacity (mut i32) (i32.const 65536))
  (global $Allocator::PAGE_SIZE i32 (i32.const 65536))

  ;; Every time heap data is requested, it will be allocated at the current bump allocator offset address,
  ;; and the offset will be incremented by the allocated amount.
  ;; This ensures that any future allocations do not overwrite existing heap data.
  ;; All allocations are 4-byte aligned.
  ;; The initial offset is not zero in order to ensure that all pointers are non-zero
  ;; (this allows cheap existence checks against unallocated memory)
  (global $Allocator::offset (mut i32) (i32.const 4))

  (func $Allocator::allocate (param $size i32) (result i32)
    ;; Reserve the requested amount of heap space and return the allocated address
    (local $target_offset i32)
    (local.set $target_offset (i32.add (global.get $Allocator::offset) (local.get $size)))
    (if
      ;; If there is insufficient spare heap capacity, allocate more pages of linear memory
      (i32.gt_u (local.get $target_offset) (global.get $Allocator::heap_capacity))
      (then
        ;; Aim to double the current heap capacity, keeping track of how many bytes were successfully allocated
        ;; (actual allocated memory can be less than the requested amount if we are approaching linear memory limits)
        (global.set $Allocator::heap_capacity
          (i32.add
            (global.get $Allocator::heap_capacity)
            (i32.mul
              (global.get $Allocator::PAGE_SIZE)
              (call $Allocator::increase_linear_memory_size (i32.div_u (global.get $Allocator::heap_capacity) (global.get $Allocator::PAGE_SIZE)))))))
      (else))
    ;; Push the existing allocator offset onto the stack (this is used as the return value)
    (global.get $Allocator::offset)
    ;; Bump the allocator offset to reserve the requested amount of heap
    (global.set $Allocator::offset (local.get $target_offset)))

  (func $Allocator::extend (param $offset i32) (param $size i32)
    ;; Extend an existing allocation by the given number of bytes
    ;; The provided offset must be the end address of the most recent allocation, or this will panic
    ;; (this is to prevent extended allocations overwriting subsequent allocations)
    (if
      (i32.ne (local.get $offset) (call $Allocator::allocate (local.get $size)))
      (then
        (unreachable))
      (else)))

  (func $Allocator::increase_linear_memory_size (param $pages i32) (result i32)
    ;; Attempt to grow the memory by the requested number of pages
    (loop $LOOP (result i32)
      (memory.grow (local.get $pages))
      ;; If the operation failed due to memory limits, we need to try again with fewer pages
      (if (result i32)
        (i32.eq (i32.const -1))
        (then
          (if (result i32)
            ;; If we failed to allocate a single page, panic
            (i32.eq (local.get $pages) (i32.const 1))
            (then
              (unreachable))
            (else
              ;; Otherwise halve the number of pages requested and try again
              (local.set $pages (i32.div_u (local.get $pages) (i32.const 2))
              (br $LOOP)))))
        (else
          ;; Return the number of pages that were successfully allocated
          (local.get $pages)))))

  (func $Allocator::pad_to_4_byte_offset (param $value i32) (result i32)
    ;; Round the given value up to the nearest 4-byte offset
    (call $Utils::i32::round_to_next (local.get $value) (i32.const 4))))
