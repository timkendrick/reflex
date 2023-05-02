;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Initialize linear memory for the heap with an initial page size of 1
  ;; (additional pages will be allocated during runtime if necessary)
  ;; https://webassembly.github.io/spec/core/exec/runtime.html#page-size
  (memory $Allocator::heap (export "memory") 1)
  (global $Allocator::PAGE_SIZE i32 (i32.const 65536))

  ;; Every time heap data is requested, it will be allocated at the current bump allocator offset address,
  ;; and the offset will be incremented by the allocated amount.
  ;; This ensures that any future allocations do not overwrite existing heap data.
  ;; All allocations are 4-byte aligned.
  ;; The current offset is stored as a 32-bit unsigned integer in the first 4 bytes of linear memory.
  ;; This is reflected in the initial offset being 4 (expressed as little-endian bytes).
  ;; This also means that an allocated pointer will never be zero, allowing cheap pointer existence checks against unallocated memory.
  (data $Allocator::heap (i32.const 0) "\04\00\00\00")

  (func $Allocator::get_offset (export "getAllocatorOffset") (result i32)
    (i32.load (i32.const 0)))

  (func $Allocator::set_offset (param $value i32)
    (i32.store (i32.const 0) (local.get $value)))

  (func $Allocator::allocate (export "allocate") (param $size i32) (result i32)
    ;; Reserve the requested amount of heap space and return the allocated address
    (local $before_offset i32)
    (local $after_offset i32)
    (local $num_existing_pages i32)
    (local $num_desired_pages i32)
    (local.set $after_offset (i32.add (local.tee $before_offset (call $Allocator::get_offset)) (local.get $size)))
    ;; Determine whether there is sufficient spare heap capacity to store the entire chunk
    (if
      (i32.lt_u
        (i32.mul (local.tee $num_existing_pages (memory.size)) (global.get $Allocator::PAGE_SIZE))
        (local.get $after_offset))
      (then
        ;; If there is insufficient spare heap capacity to store the entire chunk, we need to allocate more pages of linear memory
        ;; First determine how many extra pages are needed, doubling the current allocation until enough space has been requested
        (loop $LOOP
          ;; Loop until the requested size is sufficient to store the entire chunk
          (br_if $LOOP
            ;; Determine whether the requested size is sufficient to store the entire chunk
            (i32.lt_u
              (i32.mul
                ;; Start with a request to double the existing number of allocated pages,
                ;; continuing to double the requested size until enough space has been requested
                (local.tee $num_desired_pages
                  (select
                    ;; If this is the first iteration, start off with a request based on the number of pages already allocated
                    (select
                      ;; If there have been no heap pages allocated, start off with a request for a single page
                      (i32.const 1)
                      ;; Otherwise start off with a request to double the existing number of pages
                      (i32.mul (i32.const 2) (local.get $num_existing_pages))
                      (i32.eqz (local.get $num_existing_pages)))
                    ;; Otherwise if this is a subsequent iteration, double the requested number of pages
                    (i32.mul (i32.const 2) (local.get $num_desired_pages))
                    (i32.eqz (local.get $num_desired_pages))))
                (global.get $Allocator::PAGE_SIZE))
              (local.get $after_offset))))
        ;; Aim to allocate the requested number of pages
        ;; (actual allocated memory can be less than the requested amount if we are approaching linear memory limits)
        (call $Allocator::increase_linear_memory_size (i32.sub (local.get $num_desired_pages) (local.get $num_existing_pages)))
        ;; Add the number of allocated pages to the number of existing pages to determine the new heap size
        (i32.add (local.get $num_existing_pages))
        ;; Convert the updated heap size from pages into bytes
        (i32.mul (global.get $Allocator::PAGE_SIZE))
        ;; Determine whether the newly-allocated heap space is sufficient to perform the requested allocation
        (if
          (i32.lt_u (local.get $after_offset))
          (then
            ;; If we were unable to allocate enough heap space to perform the requested allocation, panic
            (unreachable)))))
    ;; Bump the allocator offset to reserve the requested amount of heap
    (call $Allocator::set_offset (local.get $after_offset))
    ;; Return the allocated address
    (local.get $before_offset))

  (func $Allocator::extend (param $offset i32) (param $size i32)
    ;; Extend an existing allocation by the given number of bytes
    ;; The provided offset must be the end address of the most recent allocation, or this will panic
    ;; (this is to prevent accidentally overwriting subsequent allocations)
    (if
      (i32.ne (local.get $offset) (call $Allocator::allocate (local.get $size)))
      (then
        (unreachable))
      (else)))

  (func $Allocator::shrink (export "deallocate") (param $offset i32) (param $size i32)
    ;; Shrink an existing allocation by the given number of bytes
    ;; The provided offset must be the end address of the most recent allocation, or this will panic
    ;; (this is to prevent accidentally overwriting prior allocations)
    (if
      (i32.ne (local.get $offset) (call $Allocator::get_offset))
      (then
        (unreachable))
      (else
        ;; If the request is to decrement by zero bytes, nothing more to do
        (if
          (i32.eqz (local.get $size))
          (then)
          (else
            ;; Otherwise decrement the current allocator offset by the given amount
            (call $Allocator::set_offset (local.tee $offset (i32.sub (local.get $offset) (local.get $size))))
            ;; Blank out the newly-deallocated space with zero bytes
            (memory.fill (local.get $offset) (i32.const 0) (local.get $size)))))))

  (func $Allocator::write (export "write") (param $offset i32) (param $value i32) (param $size i32)
    (i32.store (local.get $offset) (local.get $value)))

  (func $Allocator::increase_linear_memory_size (param $pages i32) (result i32)
    ;; Attempt to grow the memory by the requested number of pages
    (loop $LOOP (result i32)
      (memory.grow (local.get $pages))
      ;; If the operation failed due to memory limits, we need to try again with fewer pages
      (if (result i32)
        (i32.eq (i32.const -1))
        (then
          (if (result i32)
            ;; If we failed to allocate a single page, bail out
            (i32.eq (local.get $pages) (i32.const 1))
            (then
              (i32.const 0))
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
