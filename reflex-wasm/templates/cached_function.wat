;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (memory (import "$" "memory") 1)

  (global $NULL (import "$" "NULL") i32)

  (func $Hash::new (import "$" "createHash") (result i64))
  (func $Runtime::EvaluationCache::lookup (import "$" "getRuntimeEvaluationCacheEntry") (param $key i64) (result i32 i32))
  (func $Runtime::EvaluationCache::insert (import "$" "setRuntimeEvaluationCacheEntry") (param $key i64) (param $value i32) (param $dependencies i32) (result i32 i32))

  ;; Placeholder for the inner function that is being wrapped
  (func $__INNER (import "$" "__INNER") (; ...args ;) (param $state i32) (result i32 i32))
  ;; Placeholder for the function used to generate an invocation hash for this function being called with a specific combination of arguments
  (func $__HASH (import "$" "__HASH") (param $hash i64) (; ...args ;) (result i64))
  ;; Placeholder for a function that pushes the function arguments onto the stack
  ;; (any calls to this placeholder function will be replaced by a later compiler pass)
  (func $__ARGS (import "$" "__ARGS") (; -> ...args ;))

  (func $__template (export "main") (param $state i32) (result i32 i32)
    (local $invocation_hash i64)
    (local $value i32)
    (local $dependencies i32)
    ;; Compute the hash for this function invocation with the given arguments
    (call $Hash::new)
    (call $__ARGS)
    (call $__HASH)
    (local.set $invocation_hash)
    ;; Look up the function invocation hash in the global evaluation cache
    ;; (if an entry is found, the dependencies will include the cache dependency for this function invocation)
    (call $Runtime::EvaluationCache::lookup (local.get $invocation_hash))
    (local.set $dependencies)
    (local.tee $value)
    ;; If a cached entry was found, return the retrieved value and dependencies
    (if (result i32 i32)
      (i32.ne (global.get $NULL))
      (then
        (local.get $value)
        (local.get $dependencies))
      (else
        ;; Otherwise if there was no matching cache entry for the given function invocation,
        ;; invoke the inner function and cache the result.
        ;; Push the function invocation hash onto the stack for use later when inserting into the cache
        (local.get $invocation_hash)
        ;; Invoke the inner function
        ;; (this will leave the resulting value and dependencies on top of the stack)
        (call $__ARGS)
        (local.get $state)
        (call $__INNER)
        ;; Insert a new cache entry with the function invocation results
        ;; (this will leave the resulting value and dependencies on top of the stack,
        ;; where the dependencies will include the cache dependency for this function invocation)
        (call $Runtime::EvaluationCache::insert)))))
