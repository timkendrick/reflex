;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@include "./debugger.wat")
  (@include "./date.wat")
  (@include "./math.wat")
  (@include "./number.wat")
  (@include "./wasi.wat")
  (@include "./io.wat")
  (@include "./json.wat")
  (@include "./allocator.wat")
  (@include "./utils.wat")
  (@include "./globals.wat")
  (@include "./hash.wat")
  (@include "./cache.wat")
  (@include "./stdlib/index.wat")
  (@include "./term.wat")
  (@include "./term_type/index.wat")

  (@constructor
    (@import $Term "./term.wat"))

  (func (export "_initialize")
    (@const-init))

  (func $Runtime::get_state_value (export "getStateValue") (param $state_token i32) (param $state i32) (result i32 i32)
    ;; Retrieve the value corresponding to the provided state token from the provided state object,
    ;; or the null pointer if there is no corresponding value present in the state object
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $state))
      (then
        (global.get $NULL))
      (else
        (call $Term::Hashmap::traits::get (local.get $state) (local.get $state_token))))
    ;; Return a new dependency tree containing a single state dependency comprising the state token
    (call $Term::Tree::new
      (global.get $NULL)
      (call $Term::Dependency::state (local.get $state_token))))

  (func $Dependencies::new (export "createDependencyTree") (result i32)
    (global.get $NULL))

  (func $Dependencies::assert_empty (param $self i32)
    (if
      (i32.ne (local.get $self) (global.get $NULL))
      (then
        (unreachable))))

  (func $Dependencies::traits::union (export "combineDependencies") (param $self i32) (param $other i32) (result i32)
    (call $Term::Tree::traits::union (local.get $self) (local.get $other)))

  (func $Dependencies::get_state_value_hash (param $self i32) (param $state i32) (result i64)
    (local $state_hash i64)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $dependency i32)
    (if (result i64)
      (i32.or
        (i32.eq (global.get $NULL) (local.get $self))
        (i32.eq (global.get $NULL) (local.get $state)))
      (then
        ;; If the state is empty or the dependency list is empty, return -1
        (i64.const -1))
      (else
        (@iterate $LOOP $self $dependency $iterator_state $state $dependencies
          ;; Skip over any non-stateful dependencies
          (br_if $LOOP
            (i32.ne
              (call $Term::Dependency::get::type (local.get $dependency))
              (global.get $Dependency::StateDependency)))
          ;; Get the state value corresponding to the current dependency state token
          ;; (this will be the null pointer if no state value exists for this state token)
          (local.set $dependency
            (call $Term::Hashmap::traits::get
              (local.get $state)
              (call $Term::Dependency::StateDependency::get::condition (local.get $dependency))))
          ;; Write the state value hash (or zero if no value exists) to the combined state value hash
          (local.set $state_hash
            (call $Hash::write_i64
              (local.get $state_hash)
              (if (result i64)
                (i32.eq (local.get $dependency) (global.get $NULL))
                (then
                  (i64.const 0))
                (else
                  (call $Term::get_hash (local.get $dependency)))))))
        ;; If the state hash is unchanged (i.e. the dependency list was empty), return -1,
        ;; otherwise return the combined state value hash
        (select
          (i64.const -1)
          (local.get $state_hash)
          (i64.eq (local.get $state_hash) (call $Hash::new)))))))
