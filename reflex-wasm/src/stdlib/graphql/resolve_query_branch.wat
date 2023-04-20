;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveQueryBranch "ResolveQueryBranch"
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
        (local $dependencies i32)
        ;; Apply the query function to the current branch
        (call $Term::traits::apply
          (local.get $shape)
          (call $Term::List::of (local.get $self))
          (local.get $state))
        ;; Store the dependencies of the query function application
        (local.set $dependencies)
        ;; Evaluate all the record fields and collect into a record or signal as appropriate
        (call $Stdlib_ResolveRecord (local.get $state))
        ;; Combine the function application dependencies with the evaluation dependencies
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::Hashmap::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryBranch::impl::default (local.get $self) (local.get $shape) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::Hashset::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryBranch::impl::default (local.get $self) (local.get $shape) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::Tree::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryBranch::impl::default (local.get $self) (local.get $shape) (local.get $state))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_ResolveQueryBranch::impl::<iterate>::any (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        (local $length i32)
        (local $result i32)
        (local $item i32)
        (local $index i32)
        (local $iterator_state i32)
        (local.set $dependencies (global.get $NULL))
        ;; Create a new list that applies the query function to each source iterator item
        (@iterate-map $LOOP $self $length $result $item $index $iterator_state $state $dependencies
          ;; Apply the query function to the current item
          (call $Stdlib_ResolveQueryBranch (local.get $item) (local.get $shape) (local.get $state))
          ;; Update the accumuated dependencies
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies))))
        ;; Evaluate all the list items and collect into a list or signal as appropriate
        (call $Stdlib_CollectList (local.get $state))
        ;; Combine the accumulated iteration dependencies with the evaluation dependencies
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_ResolveQueryBranch::impl::default (param $self i32) (param $shape i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveQueryBranch)
            (call $Term::List::create_pair (local.get $self) (local.get $shape))))
        (global.get $NULL)))))
