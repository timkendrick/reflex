;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveDeep "ResolveDeep"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_ResolveDeep::impl::List (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (if (result i32 i32)
          ;; If the list is already fully resolved, return it as-is
          (call $Stdlib_ResolveDeep::is_fully_resolved::List (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the items and collect them into a new list, short-circuiting any signals
            ;; TODO: Avoid unnecessary heap allocations for intermediate values
            (local.tee $iterator
              (call $Term::MapIterator::new
                (local.get $self)
                (call $Term::Builtin::new (global.get $Stdlib_ResolveDeep))))
            (call $Term::List::traits::collect_strict (local.get $state))
            ;; Dispose the temporary iterator instance
            (call $Term::drop (local.get $iterator))))))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveDeep::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $keys i32)
        (local $values i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the record is already fully resolved, return it as-is
          (call $Stdlib_ResolveDeep::is_fully_resolved::Record (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the keys and values and create a new record, short-circuiting any signals
            (call $Stdlib_ResolveDeep::impl::List
              (call $Term::Record::get::keys (local.get $self))
              (local.get $state))
            (local.set $dependencies)
            (local.set $keys)
            (call $Stdlib_ResolveDeep::impl::List
              (call $Term::Record::get::values (local.get $self))
              (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (local.set $values)
            (if (result i32 i32)
              (i32.or
                (call $Term::Signal::is (local.get $values))
                (call $Term::Signal::is (local.get $keys)))
              (then
                (call $Term::Signal::traits::union
                  (select
                    (local.get $keys)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $keys)))
                  (select
                    (local.get $values)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $values))))
                (local.get $dependencies))
              (else
                (call $Term::Record::new (local.get $keys) (local.get $values))
                (local.get $dependencies)))))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveDeep::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (if (result i32 i32)
          ;; If the hashmap is already fully resolved, return it as-is
          (call $Stdlib_ResolveDeep::is_fully_resolved::Hashmap (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the entries and collect them into a new hashmap, short-circuiting any signals
            ;; TODO: Avoid unnecessary heap allocations for intermediate values
            (local.tee $iterator
              (call $Term::MapIterator::new
                (local.get $self)
                (call $Term::Builtin::new (global.get $Stdlib_ResolveDeep))))
            (call $Term::Hashmap::traits::collect_strict (local.get $state))
            ;; Dispose the temporary iterator instance
            (call $Term::drop (local.get $iterator))))))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_ResolveDeep::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (if (result i32 i32)
          ;; If the hashset is already fully resolved, return it as-is
          (call $Stdlib_ResolveDeep::is_fully_resolved::Hashset (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the values and collect them into a new hashset, short-circuiting any signals
            ;; TODO: Avoid unnecessary heap allocations for intermediate values
            (local.tee $iterator
              (call $Term::MapIterator::new
                (local.get $self)
                (call $Term::Builtin::new (global.get $Stdlib_ResolveDeep))))
            (call $Term::Hashset::traits::collect_strict (local.get $state))
            ;; Dispose the temporary iterator instance
            (call $Term::drop (local.get $iterator))))))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_ResolveDeep::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (local $left i32)
        (local $right i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the tree is already fully resolved, return it as-is
          (call $Stdlib_ResolveDeep::is_fully_resolved::Tree (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the child branches and create a new tree, short-circuiting any signals
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $left (call $Term::Tree::get::left (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $left) (local.get $state))
                (local.set $dependencies)
                (call $Stdlib_ResolveDeep::dispatch (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))
            (local.set $dependencies)
            (local.set $left)
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $right (call $Term::Tree::get::right (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $right) (local.get $state))
                (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
                (call $Stdlib_ResolveDeep::dispatch (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))
            (local.set $dependencies)
            (local.set $right)
            (if (result i32 i32)
              (i32.or
                (call $Term::Signal::is (local.get $right))
                (call $Term::Signal::is (local.get $left)))
              (then
                (call $Term::Signal::traits::union
                  (select
                    (local.get $left)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $left)))
                  (select
                    (local.get $right)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $right))))
                (local.get $dependencies))
              (else
                (call $Term::Tree::new (local.get $left) (local.get $right))
                (local.get $dependencies)))))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveDeep::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (local $dependencies i32)
        ;; Resolve all the items and collect them into a new list, short-circuiting any signals
        ;; TODO: Avoid unnecessary heap allocations for intermediate values
        (local.tee $iterator
          (call $Term::MapIterator::new
            (local.get $self)
            (call $Term::Builtin::new (global.get $Stdlib_ResolveDeep))))
        (call $Term::List::traits::collect_strict (local.get $state))
        ;; Dispose the temporary iterator instance
        (call $Term::drop (local.get $iterator))))

    (@default
      (func $Stdlib_ResolveDeep::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL))))


  (func $Stdlib_ResolveDeep::is_fully_resolved (param $target i32) (result i32)
    (local $term_type i32)
    (local.set $term_type (call $Term::get_type (local.get $target)))
    ;; Determine whether the item is already fully resolved according to the underlying container term type
    (@switch
      (@list
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::List))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::List (local.get $target))))
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::Record))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::Record (local.get $target))))
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::Hashmap))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::Hashmap (local.get $target))))
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::Hashset))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::Hashset (local.get $target))))
        (@list
          (i32.eq (local.get $term_type) (global.get $TermType::Tree))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::Tree (local.get $target))))
        (@list
          (call $TermType::implements::iterate (local.get $term_type))
          (return (call $Stdlib_ResolveDeep::is_fully_resolved::<iterate> (local.get $target)))))
      ;; For all other term types, refer to their underlying atomicicity value
      (call $Term::traits::is_atomic (local.get $target))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::List (param $self i32) (result i32)
    (local $length i32)
    (local $index i32)
    (if (result i32)
      ;; If the list is empty, return true
      (i32.eqz (local.tee $length (call $Term::List::get::items::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each list item in turn
        (loop $LOOP
          (if
            ;; If the current item is not fully resolved, return false
            (i32.eqz (call $Stdlib_ResolveDeep::is_fully_resolved (call $Term::List::get::items::value (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              ;; Otherwise continue with the next item
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))))
        ;; If no unresolved items were encountered in the entire list, return true
        (global.get $TRUE))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::Record (param $self i32) (result i32)
    (i32.and
      (call $Stdlib_ResolveDeep::is_fully_resolved::List (call $Term::Record::get::keys (local.get $self)))
      (call $Stdlib_ResolveDeep::is_fully_resolved::List (call $Term::Record::get::values (local.get $self)))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::Hashmap (param $self i32) (result i32)
    (local $bucket_index i32)
    (local $key i32)
    (local $capacity i32)
    (if (result i32)
      ;; If the hashmap is empty, return true
      (i32.eqz (call $Term::Hashmap::get::num_entries (local.get $self)))
      (then
        (global.get $TRUE))
      (else
        ;; Otherwise iterate through each bucket in turn
        (local.set $capacity (call $Term::Hashmap::get_capacity (local.get $self)))
        (loop $LOOP
          (if
            ;; Retrieve the bucket key and use it to determine whether the current bucket is empty
            (local.tee $key (call $Term::Hashmap::get_bucket_key (local.get $self) (local.get $bucket_index)))
            (then
              (if
                ;; If the current bucket is not empty, and its key or value is non-atomic, return false
                (i32.or
                  (i32.eqz (call $Stdlib_ResolveDeep::is_fully_resolved (local.get $key)))
                  (i32.eqz (call $Stdlib_ResolveDeep::is_fully_resolved (call $Term::Hashmap::get_bucket_value (local.get $self) (local.get $bucket_index)))))
                (then
                  (return (global.get $FALSE)))
                (else)))
            (else))
          ;; If this was not the last bucket, continue with the next bucket
          (br_if $LOOP (i32.lt_u (local.tee $bucket_index (i32.add (local.get $bucket_index) (i32.const 1))) (local.get $capacity))))
        ;; If the entire hashmap was iterated without finding a non-atomic key, return true
        (global.get $TRUE))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::Hashset (param $self i32) (result i32)
    (call $Stdlib_ResolveDeep::is_fully_resolved::Hashmap (call $Term::Hashset::get::entries (local.get $self))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::Tree (param $self i32) (result i32)
    (local $branch i32)
    (i32.and
      (if (result i32)
        (i32.eq (global.get $NULL) (local.tee $branch (call $Term::Tree::get::left (local.get $self))))
        (then
          (global.get $TRUE))
        (else
          (call $Stdlib_ResolveDeep::is_fully_resolved (local.get $branch))))
      (if (result i32)
        (i32.eq (global.get $NULL) (local.tee $branch (call $Term::Tree::get::right (local.get $self))))
        (then
          (global.get $TRUE))
        (else
          (call $Stdlib_ResolveDeep::is_fully_resolved (local.get $branch))))))

  (func $Stdlib_ResolveDeep::is_fully_resolved::<iterate> (param $self i32) (result i32)
    (global.get $FALSE)))
