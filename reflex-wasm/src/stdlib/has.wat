;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Has "Has"
    (@args (@strict $self) (@strict $key))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Has::impl::List::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (call $Term::List::traits::has (local.get $self) (local.get $key)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Has::impl::Record::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (call $Term::Record::traits::has (local.get $self) (local.get $key)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Has::impl::Hashmap::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (call $Term::Hashmap::traits::has (local.get $self) (local.get $key)))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Has::impl::Hashset::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (call $Term::Hashset::traits::has (local.get $self) (local.get $key)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Has::impl::default (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Has)
            (call $Term::List::create_pair (local.get $self) (local.get $key))))
        (global.get $NULL)))))
