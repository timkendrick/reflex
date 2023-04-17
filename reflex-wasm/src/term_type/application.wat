;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $ApplicationCache
    (@struct $ApplicationCache
      (@field $value (@ref $Term @optional))
      (@field $dependencies (@ref $Term @optional))
      (@field $overall_state_hash i64)
      (@field $minimal_state_hash i64))

    (@derive $size (@get $ApplicationCache))
    (@derive $equals (@get $ApplicationCache))
    (@derive $hash (@get $ApplicationCache))


    (@let $Application
      (@struct $Application
        (@field $target (@ref $Term))
        (@field $args (@ref $Term))
        (@field $cache (@get $ApplicationCache)))

      (@derive $size (@get $Application))
      (@derive $equals (@get $Application))

      (@export $Application (@get $Application))))


  (export "isApplication" (func $Term::Application::is))
  (export "getApplicationTarget" (func $Term::Application::get::target))
  (export "getApplicationArgs" (func $Term::Application::get::args))
  (export "getApplicationCache" (func $Term::Application::pointer::cache))

  (export "getApplicationCacheValue" (func $ApplicationCache::get::value))
  (export "getApplicationCacheDependencies" (func $ApplicationCache::get::dependencies))
  (export "getApplicationCacheOverallStateId" (func $ApplicationCache::get::overall_state_hash))
  (export "getApplicationCacheMinimalStateId" (func $ApplicationCache::get::minimal_state_hash))

  (func $Application::traits::hash (param $self i32) (param $state i64) (result i64)
    (call $Application::pointer::args (local.get $self))
    (call $Application::pointer::target (local.get $self))
    (local.get $state)
    (call $Application::hash::target)
    (call $Application::hash::args))

  (func $Application::hash::target (param $self i32) (param $state i64) (result i64)
    (call $Hash::write_i64 (local.get $state) (call $Term::get_hash (i32.load (local.get $self)))))

  (func $Application::hash::args (param $self i32) (param $state i64) (result i64)
    (call $Hash::write_i64 (local.get $state) (call $Term::get_hash (i32.load (local.get $self)))))

  (func $Term::Application::new (export "createApplication") (param $target i32) (param $args i32) (result i32)
    (local $instance i32)
    (local.tee $instance (call $Term::TermType::Application::new (local.get $target) (local.get $args)))
    (call $ApplicationCache::construct
      (call $Term::Application::pointer::cache (local.get $instance))
      (global.get $NULL)
      (global.get $NULL)
      (i64.const -1)
      (i64.const -1)))

  (func $Term::Application::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Application::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Application::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $args i32)
    (local $num_args i32)
    (local $index i32)
    ;; Write the function target to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Application::get::target (local.get $self))
        (local.get $offset)))
    ;; Write the opening parenthesis to the output
    (@store-bytes $offset "(")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the argument list to the output
    (local.set $args (call $Term::Application::get::args (local.get $self)))
    (if
      ;; If the argument list is empty, bail out
      (i32.eqz (local.tee $num_args (call $Term::List::get_length (local.get $args))))
      (then)
      (else
        ;; Otherwise iterate through each argument
        (loop $LOOP
          ;; If this is not the first argument, write a comma separator to the output
          (if
            (local.get $index)
            (then
              (@store-bytes $offset ", ")
              (local.set $offset (i32.add (local.get $offset)))))
          ;; Write the argument to the output
          (local.set $offset
            (call $Term::traits::debug
              (call $Term::List::get_item (local.get $args) (local.get $index))
              (local.get $offset)))
          ;; If this is not the final argument, continue with the next one
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_args))))))
    ;; Write the closing parenthesis to the output
    (@store-bytes $offset ")")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Return the updated offset
    (local.get $offset))

  (func $Term::Application::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Application::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Application::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_target i32)
    (local $substituted_args i32)
    (local.set $substituted_target
      (call $Term::traits::substitute
        (call $Term::Application::get::target (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_args
      (call $Term::traits::substitute
        (call $Term::Application::get::args (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_target))
        (i32.eq (global.get $NULL) (local.get $substituted_args)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Application::new
          (select
            (call $Term::Application::get::target (local.get $self))
            (local.get $substituted_target)
            (i32.eq (global.get $NULL) (local.get $substituted_target)))
          (select
            (call $Term::Application::get::args (local.get $self))
            (local.get $substituted_args)
            (i32.eq (global.get $NULL) (local.get $substituted_args)))))))

  (func $Term::Application::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Retrieve the cached value if one exists for the current state object
    (call $ApplicationCache::get_cached_value
      (call $Term::Application::pointer::cache (local.get $self))
      (local.get $state))
    ;; Pop the cached dependencies from the stack, leaving just the cached value
    (local.set $dependencies)
    (if (result i32 i32)
      ;; If a cached result exists, return the cached value and dependencies
      (i32.ne (local.tee $value) (global.get $NULL))
      (then
        (local.get $value)
        (local.get $dependencies))
      ;; Otherwise evaluate the expression and cache the result
      (else
        ;; Push the cache pointer and state pointer onto the stack (for use later when caching the evaluation result)
        (call $Term::Application::pointer::cache (local.get $self))
        (local.get $state)
        ;; Evaluate the application target
        (call $Term::traits::evaluate (call $Term::Application::get::target (local.get $self)) (local.get $state))
        ;; Pop the target dependencies from the stack, leaving just the target
        (local.set $dependencies)
        ;; Apply the target to the arguments
        (call $Term::traits::apply (call $Term::Application::get::args (local.get $self)) (local.get $state))
        ;; Pop the result dependencies and combine them with the accumulated dependencies
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; Evaluate the result
        (call $Term::traits::evaluate (local.get $state))
        ;; Pop the result dependencies and combine them with the accumulated dependencies
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        ;; Update the cached value (the instance pointer and state pointer arguments are already on the stack from
        ;; earlier in the block), leaving a copy of the value on the stack
        (call $ApplicationCache::set_cached_value (local.tee $value) (local.get $dependencies))
        ;; Push the result back onto the stack
        (local.get $value)
        ;; Push the combined dependencies onto the stack
        (local.get $dependencies))))

  (func $ApplicationCache::get_cached_value (param $self i32) (param $state i32) (result i32 i32)
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
                (call $ApplicationCache::construct
                  (local.get $self)
                  (global.get $NULL)
                  (global.get $NULL)
                  (i64.const -1)
                  (i64.const -1))
                (global.get $NULL)
                (global.get $NULL))))))))

  (func $ApplicationCache::get_state_hash (param $state i32) (result i64)
    (if (result i64)
      (i32.eq (local.get $state) (global.get $NULL))
      (then
        (i64.const -1))
      (else
        (call $Term::get_hash (local.get $state)))))

  (func $ApplicationCache::set_cached_value (param $self i32) (param $state i32) (param $value i32) (param $dependencies i32)
    (call $ApplicationCache::set::value (local.get $self) (local.get $value))
    (call $ApplicationCache::set::dependencies (local.get $self) (local.get $dependencies))
    (call $ApplicationCache::set::overall_state_hash
      (local.get $self)
      (call $ApplicationCache::get_state_hash (local.get $state)))
    (call $ApplicationCache::set::minimal_state_hash
      (local.get $self)
      ;; Compute the hash of the subset of state values that are required by the result
      (call $Dependencies::get_state_value_hash (local.get $dependencies) (local.get $state)))))
