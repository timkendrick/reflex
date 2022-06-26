;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Values
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Values::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $List::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Values::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Record::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Values::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Hashmap::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Values::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Tree::traits::values (local.get $self))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Values::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Values)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
