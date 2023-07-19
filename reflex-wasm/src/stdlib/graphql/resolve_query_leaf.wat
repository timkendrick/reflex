;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveQueryLeaf "ResolveQueryLeaf"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Nil))
      (func $Stdlib_ResolveQueryLeaf::impl::Nil (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (func $Stdlib_ResolveQueryLeaf::impl::Boolean (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_ResolveQueryLeaf::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_ResolveQueryLeaf::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ResolveQueryLeaf::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveQueryLeaf::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryLeaf::impl::default (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveQueryLeaf::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryLeaf::impl::default (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_ResolveQueryLeaf::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (call $Stdlib_ResolveQueryLeaf::impl::default (local.get $self) (local.get $state))))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_ResolveQueryLeaf::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
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
        ;; Create a new list that recursively flattens each source iterator item
        (@iterate-map $LOOP $self $length $result $item $index $iterator_state $state $dependencies
          ;; Recursively flatten the current item
          (call $Stdlib_ResolveQueryLeaf (local.get $item) (local.get $state))
          ;; Update the accumuated dependencies
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies))))
        ;; Evaluate all the list items and collect into a list or signal as appropriate
        (call $Stdlib_CollectList (local.get $state))
        ;; Combine the accumulated iteration dependencies with the evaluation dependencies
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@impl
      (call $TermType::implements::apply)
      (func $Stdlib_ResolveQueryLeaf::impl::<apply> (param $self i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; Invoke the lazy thunk function
        (call $Term::traits::apply (local.get $self) (call $Term::List::empty) (local.get $state))
        (local.set $dependencies)
        ;; Continue resolving the leaf with the thunk function return value
        (call $Stdlib_ResolveQueryLeaf (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_ResolveQueryLeaf::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveQueryLeaf)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
