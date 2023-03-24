;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectHashset "CollectHashset"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectHashset::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Hashset::traits::collect_strict (local.get $arg_list) (local.get $state))))

    (@default
      (func $Stdlib_CollectHashset::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectHashset)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
