;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Global evaluation cache

  (@const $EvaluationCache::NOT_FOUND i64 (call $Utils::i64::from_chunks (global.get $NULL) (global.get $NULL)))
  (@const-string $EvaluationCache::EFFECT_TYPE_CACHE "reflex::cache")

  ;; Pre-allocate an empty cache instance within a cell term wrapper
  (@const $EvaluationCache::EMPTY i32 (call $EvaluationCache::allocate_with_cell_wrapper (i32.const 0)))

  ;; Global hashmap pointer that stores memoized results of all function calls
  ;; (note that this points to the outer cell term wrapper, not the inner cache instance)
  (@const $EvaluationCache::INSTANCE i32
    (@depends-on $EvaluationCache::EMPTY)
    (call $Term::Pointer::new (global.get $EvaluationCache::EMPTY)))

  (export "__cache" (global $EvaluationCache::INSTANCE))

  (@let $EvaluationCache
    (@struct $EvaluationCache
      (@field $num_entries i32)
      (@field $buckets
        (@repeated
          (@struct $EvaluationCacheBucket
            (@field $key i64)
            (@field $value i64)))))

    (@derive $size (@get $EvaluationCache))

    (@constructor (@get $EvaluationCache)))

  (@apply
    (@import $HashmapMethods "./term_type/hashmap_methods.wat")
    $EvaluationCache
    $EvaluationCacheBucket
    i64
    i64
    (global.get $EvaluationCache::NOT_FOUND)
    (nop)
    (i64.eq)
    (global.get $EvaluationCache::MIN_CAPACITY)
    (call $EvaluationCache::allocate_inside_cell_wrapper)
    (nop))

  ;; Minimum hashmap capacity when dynamically allocating non-zero-length evaluation caches
  (global $EvaluationCache::MIN_CAPACITY i32 (i32.const 1024))

  (func $EvaluationCache::allocate_with_cell_wrapper (param $capacity i32) (result i32)
    (local $self i32)
    (local $inner i32)
    ;; The standard constructor wrappers take care of allocating space for a standard term,
    ;; however they do not allocate space for extra elements as needed by the cache term.
    ;; This means we have to manually allocate a larger amount of space than usual,
    ;; then fill in the cache term contents into the newly-allocated space.
    ;; We also need to additionally allocate a cell term to wrap the cache instance (the cell
    ;; wrapper is necessary because the evaluation cache type is not a term type, whereas
    ;; allocator consumers might assume all allocated values are valid terms)
    ;; First allocate a new cell term wrapper with the required capacity
    (local.tee $self
      (call $Term::Cell::allocate
        (i32.div_u
          (i32.add
            (call $EvaluationCache::sizeof)
            (i32.mul (call $EvaluationCacheBucket::sizeof) (local.get $capacity)))
          (i32.const 4))))
    ;; Then manually write the cache struct contents into the term wrapper
    (local.set $inner (call $Term::Cell::get_field_pointer (local.get $self) (i32.const 0)))
    (call $EvaluationCache::construct (local.get $inner) (i32.const 0))
    (call $EvaluationCache::set::buckets::capacity (local.get $inner) (local.get $capacity))
    (call $EvaluationCache::set::buckets::length (local.get $inner) (local.get $capacity)))

  (func $EvaluationCache::allocate_inside_cell_wrapper (param $capacity i32) (result i32)
    ;; Allocate the cell-wrapped evaluation cache instance
    (call $EvaluationCache::allocate_with_cell_wrapper (local.get $capacity))
    ;; Return a pointer to the instance within the cell wrapper
    (call $Term::Cell::get_field_pointer (i32.const 0)))

  (func $Runtime::EvaluationCache::default_capacity (param $num_entries i32) (result i32)
    ;; Low 'load factor' of 0.5 means a sparser hashmap for quicker lookups (at the expense of more space consumed)
    (i32.mul (local.get $num_entries) (i32.const 2)))

  ;; Retrieve an entry from the global cache
  (func $Runtime::EvaluationCache::lookup (export "getRuntimeEvaluationCacheEntry") (param $key i64) (result i32 i32)
    (local $cache_entry i64)
    (local $dependencies i32)
    ;; Look up the key in the global cache hashmap
    (call $EvaluationCache::retrieve
      (call $Runtime::EvaluationCache::get_instance)
      (local.get $key))
    (if (result i32 i32)
      (i64.eq (local.tee $cache_entry) (global.get $EvaluationCache::NOT_FOUND))
      (then
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Split the resulting 64-bit bucket value into a (value, dependency) tuple of two 32-bit pointers
        (call $Utils::i64::to_chunks (local.get $cache_entry))
        (local.set $dependencies)
        ;; Append the cache dependency to the retrieved cached dependencies
        (call $Term::Tree::new
          (call $Runtime::EvaluationCache::create_cache_key (local.get $key))
          (local.get $dependencies)))))

  ;; Add an entry to the global cache
  (func $Runtime::EvaluationCache::insert (export "setRuntimeEvaluationCacheEntry") (param $key i64) (param $value i32) (param $dependencies i32) (result i32 i32)
    (local $num_entries i32)
    ;; Insert the entry into the global cache hashmap
    (call $EvaluationCache::insert
      ;; Ensure enough capacity exists in the global allocator cache instance for an extra entry, reallocating if necessary
      (call $Runtime::EvaluationCache::ensure_capacity
        (call $Runtime::EvaluationCache::default_capacity
          (i32.add
            (call $EvaluationCache::get::num_entries (call $Runtime::EvaluationCache::get_instance))
            (i32.const 1))))
      (local.get $key)
      ;; Combine the two (value, dependencies) pointers into a single cache entry value
      (call $Utils::i64::from_chunks (local.get $value) (local.get $dependencies)))
      ;; Return the inserted value
      (local.get $value)
      ;; Append the cache dependency to the stored cached dependencies
      (call $Term::Tree::new
        (call $Runtime::EvaluationCache::create_cache_key (local.get $key))
        (local.get $dependencies)))

  (func $Runtime::EvaluationCache::create_cache_key (param $key i64) (result i32)
    (call $Term::Condition::custom
      (global.get $EvaluationCache::EFFECT_TYPE_CACHE)
      (call $Term::Int::new (local.get $key))
      (call $Term::Nil::new)))

  (func $Runtime::EvaluationCache::get_instance (result i32)
    (call $Term::Cell::get_field_pointer
      (call $Term::Pointer::dereference (global.get $EvaluationCache::INSTANCE))
      (i32.const 0)))

  (func $Runtime::EvaluationCache::ensure_capacity (param $capacity i32) (result i32)
    (local $existing i32)
    (local $updated i32)
    ;; Ensure enough capacity exists in the global allocator cache instance to hold an extra entry, reallocating a new
    ;; instance if necessary
    (local.tee $updated
      (call $EvaluationCache::ensure_capacity
        (local.tee $existing (call $Runtime::EvaluationCache::get_instance))
        (local.get $capacity)))
    ;; Determine whether the global cache pointer needs to be updated
    (if
      (i32.ne (local.get $existing))
      (then
        ;; If a new instance was allocated, we need to update the global cache pointer
        ;; Overwrite the global pointer term to reflect the updated address
        ;; (this ensures any existing pointers will remain valid)
        (call $Term::Pointer::construct
          (global.get $EvaluationCache::INSTANCE)
          ;; The pointer term target must be valid term, so subtract the size of the cell wrapper in order to point to
          ;; the enclosing cell term rather than the inner cache instance
          (i32.sub
            (local.get $updated)
            ;; Compute the cell header size by simulating an empty cell at offset 0 and retrieving a pointer to its contents
            (call $Term::Cell::get_field_pointer (i32.const 0) (i32.const 0))))
        ;; Reinitialize the global pointer term to recompute its hash
        (call $Term::init (global.get $EvaluationCache::INSTANCE))
        ;; Discard the pointer to the global pointer left on the stack after term (re-)initialization
        (drop)))
    ;; Return the latest instance
    (local.get $updated)))
