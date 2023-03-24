;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveList "ResolveList"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_ResolveList::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveList::impl::<iterate> (local.get $self) (local.get $state))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveList::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::List::traits::collect_strict (local.get $self) (local.get $state))))

    (@default
      (func $Stdlib_ResolveList::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveList)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
