;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveShallow "ResolveShallow"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_ResolveShallow::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveList::impl::List (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveShallow::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveRecord::impl::Record (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveShallow::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveHashmap::impl::Hashmap (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_ResolveShallow::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveHashset::impl::Hashset (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_ResolveShallow::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveTree::impl::Tree (local.get $self) (local.get $state))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveShallow::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveList::impl::<iterate> (local.get $self) (local.get $state))))

    (@default
      (func $Stdlib_ResolveShallow::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
