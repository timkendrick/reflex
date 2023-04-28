;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Global evaluation cache

  ;; Pre-allocate an empty cache instance within a cell term wrapper
  (@const $EvaluationCache::EMPTY i32 (call $EvaluationCache::allocate (i32.const 0)))

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
            (@field $value (@ref $ApplicationCache))))))

    (@derive $size (@get $EvaluationCache))

    (@constructor (@get $EvaluationCache)))

  (@apply
    (@import $HashmapMethods "./term_type/hashmap_methods.wat")
    $EvaluationCache
    $EvaluationCacheBucket
    i64
    i32
    (nop)
    (i64.eq)
    (global.get $EvaluationCache::MIN_CAPACITY)
    (call $EvaluationCache::allocate_inner)
    (nop))

  ;; Minimum hashmap capacity when dynamically allocating non-zero-length evaluation caches
  (global $EvaluationCache::MIN_CAPACITY i32 (i32.const 1024))

  (@let $ApplicationCache
    (@struct $ApplicationCache
      (@field $value (@ref $Term @optional))
      (@field $dependencies (@ref $Term @optional))
      (@field $overall_state_hash i64)
      (@field $minimal_state_hash i64))

    (@derive $size (@get $ApplicationCache))
    (@derive $equals (@get $ApplicationCache))
    (@derive $hash (@get $ApplicationCache))

    (@constructor (@get $ApplicationCache)))

  (export "getApplicationCacheValue" (func $ApplicationCache::get::value))
  (export "getApplicationCacheDependencies" (func $ApplicationCache::get::dependencies))
  (export "getApplicationCacheOverallStateId" (func $ApplicationCache::get::overall_state_hash))
  (export "getApplicationCacheMinimalStateId" (func $ApplicationCache::get::minimal_state_hash))

  (func $EvaluationCache::allocate (param $capacity i32) (result i32)
    (local $self i32)
    (local $inner i32)
    ;; The standard constructor wrappers take care of allocating space for a standard term,
    ;; however they do not allocate space for extra elements as needed by the cache term.
    ;; This means we have to manually allocate a larger amount of space than usual,
    ;; then fill in the cache term contents into the newly-allocated space.
    ;; First allocate a new cell term wrapper with the required capacity
    ;; (the cell wrapper is necessary because the evaluation cache type is not a term type, whereas
    ;; allocator consumers might assume all allocated values are valid terms)
    (local.tee $self
      (call $Term::Cell::allocate
        (i32.div_u
          (i32.add
            (call $EvaluationCache::sizeof)
            (i32.mul (call $EvaluationCacheBucket::sizeof) (local.get $capacity)))
          (i32.const 4))))
    ;; Then manually write the cache struct contents into the term wrapper
    (call $EvaluationCache::construct
      (local.tee $inner
        (call $Term::Cell::get_field_pointer
          (local.get $self)
          (i32.const 0)))
      (i32.const 0))
    (call $EvaluationCache::set::buckets::capacity (local.get $inner) (local.get $capacity))
    (call $EvaluationCache::set::buckets::length (local.get $inner) (local.get $capacity)))

  (func $EvaluationCache::allocate_inner (param $capacity i32) (result i32)
    (call $EvaluationCache::allocate (local.get $capacity))
    (call $Term::Cell::get_field_pointer (i32.const 0)))

  (func $ApplicationCache::new (export "createApplicationCache") (param $value i32) (param $dependencies i32) (param $overall_state_hash i64) (param $minimal_state_hash i64) (result i32)
    (local $instance i32)
    (local.tee $instance (call $Allocator::allocate (call $ApplicationCache::sizeof)))
    (call $ApplicationCache::construct
      (local.get $instance)
      (local.get $value)
      (local.get $dependencies)
      (local.get $overall_state_hash)
      (local.get $minimal_state_hash)))

  (func $ApplicationCache::clear (param $self i32)
    (call $ApplicationCache::construct
      (local.get $self)
      (global.get $NULL)
      (global.get $NULL)
      (i64.const -1)
      (i64.const -1)))

  (func $ApplicationCache::get_cached_value (export "retrieveApplicationCacheValue") (param $self i32) (param $state i32) (result i32 i32)
    (local $overall_state_hash i64)
    (local $minimal_state_hash i64)
    (local $cached_value i32)
    (local $cached_dependencies i32)
    (if (result i32 i32)
      ;; If there is no cached value, return the null pointer
      (i32.eq
        (local.tee $cached_value (call $ApplicationCache::get::value (local.get $self)))
        (global.get $NULL))
      (then
        (global.get $NULL)
        (global.get $NULL))
      (else
        (if (result i32 i32)
          ;; Otherwise if the current state object is identical to the cached state, return the cached value
          (i64.eq
            (local.tee $overall_state_hash (call $ApplicationCache::get_state_hash (local.get $state)))
            (call $ApplicationCache::get::overall_state_hash (local.get $self)))
          (then
            (local.get $cached_value)
            (call $ApplicationCache::get::dependencies (local.get $self)))
          (else
            ;; Otherwise determine whether the subset of required state dependency values from the cached result is
            ;; identical to the corresponding state values from the current state object
            (if (result i32 i32)
              (i64.eq
                (local.tee $minimal_state_hash
                  (call $Dependencies::get_state_value_hash
                    (local.tee $cached_dependencies (call $ApplicationCache::get::dependencies (local.get $self)))
                    (local.get $state)))
                (call $ApplicationCache::get::minimal_state_hash (local.get $self)))
              (then
                ;; If the subset of required state dependency values is unchanged,
                ;; update the overall state hash and return the cached result
                (call $ApplicationCache::set::overall_state_hash (local.get $self) (local.get $overall_state_hash))
                (local.get $cached_value)
                (local.get $cached_dependencies))
              (else
                ;; Otherwise clear the cached result and return the null pointer
                ;; (we clear the cached result due to the assumption that the state advances monotonically,
                ;; making it pointless to retain outdated values that are unlikely to become valid again)
                (call $ApplicationCache::clear (local.get $self))
                (global.get $NULL)
                (global.get $NULL))))))))

  (func $ApplicationCache::set_cached_value (export "updateApplicationCacheEntry") (param $self i32) (param $state i32) (param $value i32) (param $dependencies i32)
    (call $ApplicationCache::construct
      (local.get $self)
      (local.get $value)
      (local.get $dependencies)
      (call $ApplicationCache::get_state_hash (local.get $state))
      ;; Compute the hash of the subset of state values that are required by the result
      (call $Dependencies::get_state_value_hash (local.get $dependencies) (local.get $state))))

  (func $ApplicationCache::get_state_hash (param $state i32) (result i64)
    (if (result i64)
      (i32.eq (local.get $state) (global.get $NULL))
      (then
        (i64.const -1))
      (else
        (call $Term::get_hash (local.get $state)))))

  (func $Runtime::EvaluationCache::default_capacity (param $num_entries i32) (result i32)
    ;; Low 'load factor' of 0.5 means a sparser hashmap for quicker lookups (at the expense of more space consumed)
    (i32.mul (local.get $num_entries) (i32.const 2)))

  ;; Retrieve an entry from the global cache
  (func $Runtime::EvaluationCache::lookup (export "getRuntimeEvaluationCacheEntry") (param $key i64) (result i32)
    ;; Look up the key in the global cache hashmap
    (call $EvaluationCache::retrieve
      (call $Runtime::EvaluationCache::get_instance)
      (local.get $key)))

  ;; Add an entry to the global cache
  (func $Runtime::EvaluationCache::insert (export "setRuntimeEvaluationCacheEntry") (param $key i64) (param $state i32) (param $value i32) (param $dependencies i32)
    (local $num_entries i32)
    (local $cache_entry i32)
    ;; Allocate a new application cache entry (note that this is NOT a valid term as it does not have a term wrapper)
    (local.set $cache_entry
      (call $ApplicationCache::new
        (local.get $value)
        (local.get $dependencies)
        (call $ApplicationCache::get_state_hash (local.get $state))
        ;; Compute the hash of the subset of state values that are required by the result
        (call $Dependencies::get_state_value_hash (local.get $dependencies) (local.get $state))))
    ;; Insert the entry into the global cache hashmap
    (call $EvaluationCache::insert
      ;; Ensure enough capacity exists in the global allocator cache instance for an extra entry, reallocating if necessary
      (call $Runtime::EvaluationCache::ensure_capacity
        (call $Runtime::EvaluationCache::default_capacity
          (i32.add
            (call $EvaluationCache::get::num_entries (call $Runtime::EvaluationCache::get_instance))
            (i32.const 1))))
      (local.get $key)
      (local.get $cache_entry)))

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
