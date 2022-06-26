;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Keys
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Keys::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $List::traits::keys (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Keys::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Record::traits::keys (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Keys::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Hashmap::traits::keys (local.get $self))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Keys::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Keys)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
