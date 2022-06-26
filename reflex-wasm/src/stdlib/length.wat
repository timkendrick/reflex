;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Length
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Length::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Int::new (call $List::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Length::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Int::new (call $Record::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Length::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Int::new (call $Hashmap::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Length::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Int::new (call $Tree::traits::length (local.get $self)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Length::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Length)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
