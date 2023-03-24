;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Unzip "Unzip"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Unzip::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::List::create_pair
          (call $Term::HashmapKeysIterator::new (local.get $self))
          (call $Term::HashmapValuesIterator::new (local.get $self)))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_Unzip::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::List::create_pair
          (call $Term::IndexedAccessorIterator::new (local.get $self) (i32.const 0))
          (call $Term::IndexedAccessorIterator::new (local.get $self) (i32.const 1)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Unzip::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Unzip)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
