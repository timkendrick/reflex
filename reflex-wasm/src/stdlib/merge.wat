;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Merge "Merge"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_Merge::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $entries i32)
        (local $keys i32)
        (local $values i32)
        (local $num_entries i32)
        (local $dependencies i32)
        ;; Combine all the sets of record entries into a temporary lookup hashmap instance
        (call $Term::Hashmap::traits::collect
          (call $Term::FlattenIterator::new (call $Term::FlattenIterator::new (local.get $arg_list)))
          (local.get $state))
        (local.set $dependencies)
        (local.tee $entries)
        (if (result i32 i32)
          (i32.eqz (local.tee $num_entries (call $Term::Hashmap::get::num_entries)))
          (then
            ;; If no entries were produced, return the pre-allocated singleton instance
            ;; Dispose of the temporary lookup hashmap (this is effectively a no-op as the empty hashmap cannot be dropped)
            (call $Term::Hashmap::drop (local.get $entries))
            (return
              (call $Term::Record::empty)
              (global.get $NULL)))
          (else
            ;; Otherwise collect lists for keys and values
            (call $Term::List::traits::collect
              (call $Term::HashmapKeysIterator::new (local.get $entries))
              (global.get $NULL))
            (call $Dependencies::assert_empty)
            (local.set $keys)
            (call $Term::List::traits::collect
              (call $Term::HashmapValuesIterator::new (local.get $entries))
              (global.get $NULL))
            (call $Dependencies::assert_empty)
            (local.set $values)
            ;; Construct the record object
            (call $Term::TermType::Record::new
              (local.get $keys)
              (local.get $values)
              ;; Determine whether to use the lookup hashmap depending on the number of record fields
              (if (result i32)
                (i32.ge_u (local.get $num_entries) (global.get $Term::Record::LOOKUP_TABLE_MIN_SIZE))
                (then
                  (local.get $entries))
                (else
                  ;; Dispose of the temporary lookup hashmap
                  (call $Term::Hashmap::drop (local.get $entries))
                  (global.get $NULL))))
            (local.get $dependencies)))))

    (@default
      (func $Stdlib_Merge::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Merge)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
