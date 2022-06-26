;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Values
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Values::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::List::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Values::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Record::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Values::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Hashmap::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_Values::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Hashset::traits::values (local.get $self))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Values::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Tree::traits::values (local.get $self))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Values::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Values)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
