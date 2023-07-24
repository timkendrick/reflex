;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectConstructor "CollectConstructor"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_CollectConstructor::impl::List (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Constructor::new (local.get $arg_list))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectConstructor::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; TODO: Avoid unnecessary heap allocations for intermediate values
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (call $Stdlib_CollectConstructor::impl::List (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_CollectConstructor::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectConstructor)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
