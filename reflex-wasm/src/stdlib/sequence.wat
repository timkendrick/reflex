;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Sequence "Sequence"
    (@args (@strict $self) (@strict $callback))

    (@impl
      (i32.or (i32.const 0xFFFFFFFF))
      (call $TermType::implements::apply)
      (func $Stdlib_Sequence::impl::any::<apply> (param $self i32) (param $callback i32) (param $state i32) (result i32 i32)
        (call $Term::traits::apply (local.get $callback) (call $Term::List::of (local.get $self)) (local.get $state))))

    (@default
      (func $Stdlib_Sequence::impl::default (param $self i32) (param $callback i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
