;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Hash "Hash"
    (@args (@variadic (@strict $arg_list)))

    (@default
      (func $Stdlib_Hash::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Symbol::new
          ;; TODO: Confirm conversion of 64-bit hash to 32-bit symbol ID
          (i32.wrap_i64 (call $Term::get_hash (local.get $arg_list))))
        (global.get $NULL)))))
