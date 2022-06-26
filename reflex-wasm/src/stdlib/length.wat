;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Length
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Length::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (call $Term::List::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Length::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (call $Term::Record::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Length::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (call $Term::Hashmap::traits::length (local.get $self)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Length::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (call $Term::Tree::traits::length (local.get $self)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Length::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Length)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
