;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_ResolveShallow
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_ResolveShallow::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (if (result i32 i32)
          ;; If the list is already fully resolved, return it as-is
          (call $List::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the items and collect them into a new list, short-circuiting any signals
            (call $List::traits::collect_strict
              ;; TODO: Avoid unnecessary heap allocations for intermediate values
              (call $EvaluateIterator::new (local.get $self))
              (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveShallow::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $keys i32)
        (local $values i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the record is already fully resolved, return it as-is
          (call $Record::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the keys and values and create a new record, short-circuiting any signals
            (call $Stdlib_ResolveShallow::impl::List
              (call $Record::get::keys (local.get $self))
              (local.get $state))
            (local.set $dependencies)
            (local.set $keys)
            (call $Stdlib_ResolveShallow::impl::List
              (call $Record::get::values (local.get $self))
              (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (local.set $values)
            (if (result i32 i32)
              (i32.or
                (call $Signal::is (local.get $values))
                (call $Signal::is (local.get $keys)))
              (then
                (call $Signal::traits::union
                  (select
                    (local.get $keys)
                    (global.get $NULL)
                    (call $Signal::is (local.get $keys)))
                  (select
                    (local.get $values)
                    (global.get $NULL)
                    (call $Signal::is (local.get $values))))
                (local.get $dependencies))
              (else
                (call $Record::new (local.get $keys) (local.get $values))
                (local.get $dependencies)))))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveShallow::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (if (result i32 i32)
          ;; If the hashmap is already fully resolved, return it as-is
          (call $Hashmap::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the entries and collect them into a new hashmap, short-circuiting any signals
            (call $Hashmap::traits::collect_strict
              ;; TODO: Avoid unnecessary heap allocations for intermediate values
              (call $ZipIterator::new
                (call $EvaluateIterator::new
                  (call $HashmapKeysIterator::new (local.get $self)))
                (call $EvaluateIterator::new
                  (call $HashmapValuesIterator::new (local.get $self))))
              (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_ResolveShallow::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (local $left i32)
        (local $right i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the tree is already fully resolved, return it as-is
          (call $Tree::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the child branches and create a new tree, short-circuiting any signals
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $left (call $Tree::get::left (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $left) (local.get $state))))
            (local.set $dependencies)
            (local.set $left)
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $right (call $Tree::get::right (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $right) (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))
            (local.set $dependencies)
            (local.set $right)
            (if (result i32 i32)
              (i32.or
                (call $Signal::is (local.get $right))
                (call $Signal::is (local.get $left)))
              (then
                (call $Signal::traits::union
                  (select
                    (local.get $left)
                    (global.get $NULL)
                    (call $Signal::is (local.get $left)))
                  (select
                    (local.get $right)
                    (global.get $NULL)
                    (call $Signal::is (local.get $right))))
                (local.get $dependencies))
              (else
                (call $Tree::new (local.get $left) (local.get $right))
                (local.get $dependencies)))))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveShallow::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $items i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the iterator is already fully resolved, return it as-is
          (call $Term::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the items and collect them into a new list, short-circuiting any signals
            (call $List::traits::collect_strict
              ;; TODO: Avoid unnecessary heap allocations for intermediate values
              (call $EvaluateIterator::new (local.get $self))
              (local.get $state))))))

    (@default
      (func $Stdlib_ResolveShallow::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
