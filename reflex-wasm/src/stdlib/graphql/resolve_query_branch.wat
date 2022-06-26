;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveQueryBranch
    (@args (@strict $self) (@strict $shape))

    (@impl
      (i32.eq (global.get $TermType::Nil))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::Nil::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::Record::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryBranch::impl::default (local.get $self) (local.get $shape) (local.get $state))))

    (@impl
      (call $Term::implements::iterate)
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::<iterate>::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        (local $length i32)
        (local $result i32)
        (local $item i32)
        (local $index i32)
        (local $iterator_state i32)
        (local.set $dependencies (global.get $NULL))
        ;; Recursively resolve each source iterator item
        (@iterate-map $self $length $result $item $index $iterator_state $state $dependencies
          (call $Stdlib_ResolveQueryBranch::call_static (local.get $item) (local.get $shape) (local.get $state)))))

    (@default
      (func $Stdlib_ResolveQueryBranch::impl::default (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; Apply the query shape function to the current branch
        (call $Term::traits::apply
          (local.get $shape)
          (call $Term::List::of (local.get $self))
          (local.get $state))
        ;; Store the dependencies of the function application
        (local.set $dependencies)
        ;; Evaluate the function application result
        (call $Term::traits::evaluate (local.get $state))
        ;; Combine the stored function application dependencies with the result evaluation dependencies
        (call $Dependencies::traits::union (local.get $dependencies))))))
