;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@include "./math.wat")
  (@include "./wasi.wat")
  (@include "./io.wat")
  (@include "./json.wat")
  (@include "./allocator.wat")
  (@include "./utils.wat")
  (@include "./globals.wat")
  (@include "./hash.wat")
  (@include "./stdlib/index.wat")
  (@include "./term.wat")
  (@include "./term/index.wat")

  (@constructor
    (@import $Term "./term.wat"))

  (func (export "_initialize")
    (call $Io::startup)
    ;; Invoke any startup hooks defined by the term type implementations
    (call $Term::startup))

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
    (call $Term::Tree::traits::union (local.get $self) (local.get $other))))
