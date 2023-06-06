;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Length "Length"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Length::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::String::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Length::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::List::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Length::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::Record::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_Length::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::Hashmap::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_Length::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::Hashset::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Length::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new (i64.extend_i32_u (call $Term::Tree::traits::length (local.get $self))))
        (global.get $NULL)))

    (@impl
      (call $Term::traits::iterate)
      (func $Stdlib_Length::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $size_hint i32)
        (local $iterator_state i32)
        (local $dependencies i32)
        (local $item i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $size_hint (call $Term::traits::size_hint (local.get $self)))
            (global.get $NULL))
          (then
            (call $Term::Int::new (i64.extend_i32_u (local.get $size_hint)))
            (global.get $NULL))
          (else
            (local.set $size_hint (i32.const 0))
            (local.set $dependencies (global.get $NULL))
            (@iterate $self $item $iterator_state $state $dependencies
              (local.set $size_hint (i32.add (i32.const 1) (local.get $size_hint))))
            (call $Term::Int::new (i64.extend_i32_u (local.get $size_hint)))
            (local.get $dependencies)))))

    (@default
      (func $Stdlib_Length::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Length)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
