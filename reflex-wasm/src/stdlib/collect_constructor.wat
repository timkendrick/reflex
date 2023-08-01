;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectConstructor "CollectConstructor"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_CollectConstructor::impl::List (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $num_entries i32)
        (local $lookup i32)
        (local $num_deduplicated_entries i32)
        (local $keys i32)
        (local $index i32)
        (local $key i32)
        (local $insertion_index i32)
        (if (result i32 i32)
          ;; If the argument list is empty, return the empty constructor
          (i32.eqz
            (local.tee $num_entries (call $Term::List::get_length (local.get $arg_list))))
          (then
            (call $Term::Constructor::empty)
            (global.get $NULL))
          (else
            ;; Collect the provided keys into a lookup hashmap that can be used to test for the existence of keys
            (call $Term::Hashmap::traits::collect
              (call $Term::FlattenIterator::new
                (call $Term::ZipIterator::new
                  (local.get $arg_list)
                  (local.get $arg_list)))
              (local.get $state))
            (call $Dependencies::assert_empty)
            (local.tee $lookup)
            ;; Determine the number of deduplicated key entries that were provided
            (local.set $num_deduplicated_entries (call $Term::Hashmap::traits::length))
            ;; Allocate a new list to hold the deduplicated constructor keys
            (local.set $keys (call $Term::List::allocate (local.get $num_deduplicated_entries)))
            ;; Populate the constructor keys either directly from the argument list or from the keys lookup
            ;; according to whether duplicate keys were provided
            (if (result i32 i32)
              (i32.eq (local.get $num_entries) (local.get $num_deduplicated_entries))
              (then
                ;; If there were no duplicate keys provided, iterate over every item in the argument list,
                ;; populating the keys list by copying the corresponding item from the argument list
                (loop $LOOP
                  ;; Copy the key from the argument list
                  (call $Term::List::set_item
                    (local.get $keys)
                    (local.get $index)
                    (call $Term::List::get_item (local.get $arg_list) (local.get $index)))
                  ;; If there are more remaining keys to process, continue with the next item in the argument list
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                      (local.get $num_entries))))
                ;; Dispose the temporary lookup table
                (call $Term::Hashmap::drop (local.get $lookup))
                ;; Return a new constructor instance with the newly-initialized keys
                (call $Term::TermType::Constructor::new
                  ;; Initialize the keys list
                  (call $Term::List::init (local.get $keys) (local.get $num_entries)))
                (global.get $NULL))
              (else
                ;; Otherwise if there were duplicate keys provided, iterate over the keys in the arguments list,
                ;; populating the keys list by copying the argument list item if present in the lookup table and
                ;; removing entries from the lookup table as they are inserted into the list
                (loop $LOOP
                  ;; Get the key from the argument list
                  (local.set $key
                    (call $Term::List::get_item (local.get $arg_list) (local.get $index)))
                  ;; Look up the corresponding value from the lookup table
                  (call $Term::Hashmap::traits::get (local.get $lookup) (local.get $key))
                  ;; Increment the iteration index in preparation for the next iteration
                  (local.set $index (i32.add (i32.const 1) (local.get $index)))
                  ;; If no lookup entry exists for this key (due to having already been processed and removed in a
                  ;; previous iteration), continue with the next entry in the argument list
                  (br_if $LOOP (i32.eq (global.get $NULL)))
                  ;; Remove the lookup entry for this key (this ensures each key is only processed once)
                  (call $Term::Hashmap::delete (local.get $lookup) (local.get $key))
                  ;; Copy the key into the next free index in the deduplicated key lue lists
                  ;; (this ensures original key order is maintained)
                  (call $Term::List::set_item (local.get $keys) (local.get $insertion_index) (local.get $key))
                  ;; Increment the insertion index for the next deduplicated entry,
                  ;; and continue with the next entry if all entries have not yet been processed
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $insertion_index (i32.add (i32.const 1) (local.get $insertion_index)))
                      (local.get $num_deduplicated_entries))))
                ;; Dispose the temporary lookup table
                ;; (this will have been fully emptied over the course of the iteration)
                (call $Term::Hashmap::drop (local.get $lookup))
                ;; Return a new constructor instance with the newly-initialized keys and values
                (call $Term::Constructor::new
                  ;; Initialize the keys list
                  (call $Term::List::init (local.get $keys) (local.get $num_deduplicated_entries)))
                (global.get $NULL)))))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectConstructor::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; TODO: Avoid unnecessary heap allocations for intermediate values
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (call $Stdlib_CollectConstructor::impl::List (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_CollectConstructor::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectConstructor)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
