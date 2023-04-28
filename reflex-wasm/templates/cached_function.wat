;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (memory (import "$" "memory") 1)

  (global $NULL (import "$" "NULL") i32)

  (func $Hash::new (import "$" "createHash") (result i64))
  (func $Runtime::EvaluationCache::lookup (import "$" "getRuntimeEvaluationCacheEntry") (param $key i64) (result i32))
  (func $Runtime::EvaluationCache::insert (import "$" "setRuntimeEvaluationCacheEntry") (param $key i64) (param $state i32) (param $value i32) (param $dependencies i32))
  (func $ApplicationCache::get_cached_value (import "$" "retrieveApplicationCacheValue") (param $cache_entry i32) (param $state i32) (result i32 i32))
  (func $ApplicationCache::set_cached_value (import "$" "updateApplicationCacheEntry") (param $cache_entry i32) (param $state i32) (param $value i32) (param $dependencies i32))

  ;; Placeholder for the inner function that is being wrapped
  (func $__INNER (import "$" "__INNER") (; ...args ;) (param $state i32) (result i32 i32))
  ;; Placeholder for the function used to generate an invocation hash for this function being called with a specific combination of arguments
  (func $__HASH (import "$" "__HASH") (param $hash i64) (; ...args ;) (result i64))
  ;; Placeholder for a function that pushes the function arguments onto the stack
  ;; (any calls to this placeholder function will be replaced by a later compiler pass)
  (func $__ARGS (import "$" "__ARGS") (; -> ...args ;))

  (func $__template (export "main") (param $state i32) (result i32 i32)
    (local $invocation_hash i64)
    (local $cache_entry i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Compute the hash for this function invocation with the given arguments
    (call $Hash::new)
    (call $__ARGS)
    (call $__HASH)
    (local.set $invocation_hash)
    ;; Look up the function invocation hash in the global evaluation cache
    (local.tee $cache_entry (call $Runtime::EvaluationCache::lookup (local.get $invocation_hash)))
    ;; If a cache entry exists, determine whether the result is valid for the current state
    (if (result i32 i32)
      (i32.ne (global.get $NULL))
      (then
        ;; Retrieve the value and dependencies if the entry is valid for this state, or null pointers if not
        (call $ApplicationCache::get_cached_value (local.get $cache_entry) (local.get $state))
        (local.set $dependencies)
        (local.tee $value)
        ;; Determine whether a valid cached value was retrieved for the given function invocation hash
        (if (result i32 i32)
          (i32.ne (global.get $NULL))
          (then
            ;; If a valid result was retrieved, return the cached value and dependencies
            (local.get $value)
            (local.get $dependencies))
          (else
            ;; Otherwise if the cache entry is not valid for this state, invoke the inner function
            (call $__ARGS)
            (local.get $state)
            (call $__INNER)
            (local.set $dependencies)
            (local.set $value)
            ;; Update the cache entry with the function invocation results
            (call $ApplicationCache::set_cached_value
              (local.get $cache_entry)
              (local.get $state)
              (local.get $value)
              (local.get $dependencies))
            ;; Return the evaluation result
            (local.get $value)
            (local.get $dependencies))))
      (else
        ;; Otherwise if there was no matching cache entry for the given function, invoke the inner function
        (call $__ARGS)
        (local.get $state)
        (call $__INNER)
        (local.set $dependencies)
        (local.set $value)
        ;; Insert a new cache entry with the function invocation results
        (call $Runtime::EvaluationCache::insert
          (local.get $invocation_hash)
          (local.get $state)
          (local.get $value)
          (local.get $dependencies))
        ;; Return the evaluation result
        (local.get $value)
        (local.get $dependencies)))))
