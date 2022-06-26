;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectHashmap
    (@args (@strict $self))

    (@impl
      (call $Term::implements::iterate)
      (func $Stdlib_CollectHashmap::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Hashmap::traits::collect_strict (local.get $self) (local.get $state))))

    (@default
      (func $Stdlib_CollectHashmap::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectHashmap)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
