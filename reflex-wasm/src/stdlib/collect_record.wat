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
        (local $keys i32)
        (local $values i32)
        (local $index i32)
        (if (result i32 i32)
          ;; If the argument list is empty, return the empty record
          (i32.eqz
            (local.tee $num_entries
              (i32.div_u (call $Term::List::get_length (local.get $arg_list)) (i32.const 2))))
          (then
            (call $Term::Record::empty)
            (global.get $NULL))
          (else
            ;; Otherwise allocate new keys and values lists to hold the record entries
            (local.set $keys (call $Term::List::allocate (local.get $num_entries)))
            (local.set $values (call $Term::List::allocate (local.get $num_entries)))
            ;; Iterate over the argument list, populating the keys and values lists
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
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                  (local.get $num_entries))))
            ;; Initialize the keys and values lists
            (call $Term::List::init (local.get $keys) (local.get $num_entries))
            (call $Term::List::init (local.get $values) (local.get $num_entries))
            ;; Return a new record term instance with the newly-initialized keys and values
            (call $Term::Record::new)
            (global.get $NULL)))))

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
