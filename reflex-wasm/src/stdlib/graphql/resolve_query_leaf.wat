;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveQueryLeaf
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveQueryLeaf::impl::Record::any (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryLeaf::impl::default (local.get $self) (local.get $state))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveQueryLeaf::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        (local $length i32)
        (local $result i32)
        (local $item i32)
        (local $index i32)
        (local $iterator_state i32)
        (local.set $dependencies (global.get $NULL))
        ;; Recursively flatten each source iterator item
        (@iterate-map $self $length $result $item $index $iterator_state $state $dependencies
          (call $Stdlib_ResolveQueryLeaf::call_static (local.get $item) (local.get $state)))))

    (@default
      (func $Stdlib_ResolveQueryLeaf::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
