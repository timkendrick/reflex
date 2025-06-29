;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectTree "CollectTree"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectTree::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Tree::traits::collect (local.get $arg_list) (local.get $state))))

    (@default
      (func $Stdlib_CollectTree::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectTree)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
