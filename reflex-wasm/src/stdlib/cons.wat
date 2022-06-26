;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Cons
    (@args (@lazy $head) (@lazy $tail))

    (@default
      (func $Stdlib_Cons::impl::default (param $head i32) (param $tail i32) (param $state i32) (result i32 i32)
        (call $Tree::new (local.get $head) (local.get $tail))
        (global.get $NULL)))))
