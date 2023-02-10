;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
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
  (@include "./stdlib/index.wat")
  (@include "./term.wat")
  (@include "./term_type/index.wat")

  (@constructor
    (@import $Term "./term.wat"))

  (func (export "_initialize")
    (@const-init))

  (func $Runtime::get_state_value (param $state_token i32) (param $state i32) (result i32 i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $state))
      (then
        (global.get $NULL))
      (else
        (call $Term::Hashmap::traits::get (local.get $state) (local.get $state_token))))
    (call $Dependencies::of (local.get $state_token)))

  (func $Dependencies::of (param $state_token i32) (result i32)
    (call $Term::Tree::of (local.get $state_token)))

  (func $Dependencies::traits::union (param $self i32) (param $other i32) (result i32)
    (call $Term::Tree::traits::union (local.get $self) (local.get $other)))

  (func $Dependencies::get_state_value_hash (param $self i32) (param $state i32) (result i32)
    (local $state_hash i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $item i32)
    (if (result i32)
      (i32.or
        (i32.eq (global.get $NULL) (local.get $self))
        (i32.eq (global.get $NULL) (local.get $state)))
      (then
        (global.get $NULL))
      (else
        (@iterate $self $item $iterator_state $state $dependencies
          ;; Get the state value corresponding to the current dependency state token
          ;; (this will be the null pointer if no state value exists for this state token)
          (local.set $item (call $Term::Hashmap::traits::get (local.get $state) (local.get $item)))
          ;; Write the state value hash (or null if no value exists) to the combined state value hash
          (local.set $state_hash
            (call $Hash::write_i32
              (local.get $state_hash)
              (if (result i32)
                (i32.eq (local.get $item) (global.get $NULL))
                (then
                  (global.get $NULL))
                (else
                  (call $Term::get_hash (local.get $item)))))))
        ;; If the state hash is unchanged (i.e. the dependency list was empty), return the null pointer,
        ;; otherwise return the combined state value hash
        (select
          (global.get $NULL)
          (local.get $state_hash)
          (i32.eq (local.get $state_hash) (call $Hash::new)))))))
