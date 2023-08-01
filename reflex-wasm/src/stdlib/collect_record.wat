;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectRecord "CollectRecord"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_CollectRecord::impl::List (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $num_entries i32)
        (local $lookup i32)
        (local $num_deduplicated_entries i32)
        (local $keys i32)
        (local $values i32)
        (local $index i32)
        (local $key i32)
        (local $value i32)
        (local $insertion_index i32)
        (if (result i32 i32)
          ;; If the argument list is empty, return the empty record
          (i32.eqz
            (local.tee $num_entries
              (i32.div_u (call $Term::List::get_length (local.get $arg_list)) (i32.const 2))))
          (then
            (call $Term::Record::empty)
            (global.get $NULL))
          (else
            ;; Collect the provided keys and values into a lookup hashmap that maps keys to the corresponding values
            (call $Term::Hashmap::traits::collect (local.get $arg_list) (local.get $state))
            (call $Dependencies::assert_empty)
            (local.tee $lookup)
            ;; Determine the number of deduplicated key entries that were provided
            (local.set $num_deduplicated_entries (call $Term::Hashmap::traits::length))
            ;; Allocate new lists to hold the deduplicated record keys and values
            (local.set $keys (call $Term::List::allocate (local.get $num_deduplicated_entries)))
            (local.set $values (call $Term::List::allocate (local.get $num_deduplicated_entries)))
            ;; Populate the record entries either directly from the argument list or from the values lookup
            ;; according to whether duplicate keys were provided
            (if (result i32 i32)
              (i32.eq (local.get $num_entries) (local.get $num_deduplicated_entries))
              (then
                ;; If there were no duplicate keys provided, iterate over every other item in the argument list,
                ;; populating the keys and values lists by copying the corresponding items from the argument list
                (loop $LOOP
                  ;; Copy the key from the argument list
                  (call $Term::List::set_item
                    (local.get $keys)
                    (local.get $index)
                    (call $Term::List::get_item
                      (local.get $arg_list)
                      (i32.add (i32.mul (local.get $index) (i32.const 2)) (i32.const 0))))
                  ;; Copy the value from the argument list
                  (call $Term::List::set_item
                    (local.get $values)
                    (local.get $index)
                    (call $Term::List::get_item
                      (local.get $arg_list)
                      (i32.add (i32.mul (local.get $index) (i32.const 2)) (i32.const 1))))
                  ;; If there are more remaining entries to process, continue with the next entry in the argument list
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                      (local.get $num_entries))))
                ;; Return a new record term instance with the newly-initialized keys and values
                (call $Term::TermType::Record::new
                  ;; Initialize the keys list
                  (call $Term::List::init (local.get $keys) (local.get $num_entries))
                  ;; Initialize the values list
                  (call $Term::List::init (local.get $values) (local.get $num_entries))
                  ;; Determine whether to assign the lookup hashmap depending on the number of record fields
                  (if (result i32)
                    (i32.ge_u (local.get $num_entries) (global.get $Term::Record::LOOKUP_TABLE_MIN_SIZE))
                    (then
                      ;; If there are enough entries to justify a lookup hashmap, use the prepopulated lookup table
                      (local.get $lookup))
                    (else
                      ;; Otherwise dispose the temporary lookup table
                      (call $Term::Hashmap::drop (local.get $lookup))
                      (global.get $NULL))))
                (global.get $NULL))
              (else
                ;; Otherwise if there were duplicate keys provided, iterate over the keys in the arguments list,
                ;; populating the keys list by copying the corresponding item from the argument list and
                ;; populating the values list by looking up the value corresponding to the key in the lookup table,
                ;; removing entries from the lookup table as they are inserted into the list
                (loop $LOOP
                  ;; Get the key from the argument list
                  (local.set $key
                    (call $Term::List::get_item
                      (local.get $arg_list)
                      (i32.mul (local.get $index) (i32.const 2))))
                  ;; Look up the corresponding value from the lookup table
                  (local.tee $value (call $Term::Hashmap::traits::get (local.get $lookup) (local.get $key)))
                  ;; Increment the iteration index in preparation for the next iteration
                  (local.set $index (i32.add (i32.const 1) (local.get $index)))
                  ;; If no lookup entry exists for this key (due to having already been processed and removed in a
                  ;; previous iteration), continue with the next entry in the argument list
                  (br_if $LOOP (i32.eq (global.get $NULL)))
                  ;; Remove the lookup entry for this key (this ensures each key is only processed once)
                  (call $Term::Hashmap::delete (local.get $lookup) (local.get $key))
                  ;; Copy the key and value into the next free index in the deduplicated key and value lists
                  ;; (this ensures original key order is maintained)
                  (call $Term::List::set_item (local.get $keys) (local.get $insertion_index) (local.get $key))
                  (call $Term::List::set_item (local.get $values) (local.get $insertion_index) (local.get $value))
                  ;; Increment the insertion index for the next deduplicated entry,
                  ;; and continue with the next entry if all entries have not yet been processed
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $insertion_index (i32.add (i32.const 1) (local.get $insertion_index)))
                      (local.get $num_deduplicated_entries))))
                ;; Dispose the temporary lookup table
                ;; (this will have been fully emptied over the course of the iteration)
                (call $Term::Hashmap::drop (local.get $lookup))
                ;; Return a new record term instance with the newly-initialized keys and values
                (call $Term::Record::new
                  ;; Initialize the keys list
                  (call $Term::List::init (local.get $keys) (local.get $num_deduplicated_entries))
                  ;; Initialize the values list
                  (call $Term::List::init (local.get $values) (local.get $num_deduplicated_entries)))
                (global.get $NULL)))))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectRecord::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        ;; TODO: Avoid unnecessary heap allocations for intermediate values
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (call $Stdlib_CollectRecord::impl::List (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_CollectRecord::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectRecord)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
