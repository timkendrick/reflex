;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_CollectTree
    (@args (@strict $self))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectTree::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Tree::traits::collect_strict (local.get $self) (local.get $state))))

    (@default
      (func $Stdlib_CollectTree::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectTree)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
