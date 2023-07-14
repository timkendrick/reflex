;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectString "CollectString"
    (@args (@variadic (@strict $arg_list)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_CollectString::impl::List (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $num_items i32)
        (local $index i32)
        ;; Determine whether the argument list is empty
        (if
          (i32.eqz (local.tee $num_items (call $Term::List::get_length (local.get $arg_list))))
          (then
            ;; If the argument list is empty, return the empty string
            (return
              (call $Term::String::empty)
              (global.get $NULL)))
          (else
            ;; Iterate through the arguments to assert that they are all strings
            (loop $LOOP
              ;; Determine whether the argument is a string
              (if
                (i32.eqz (call $Term::String::is (call $Term::List::get_item (local.get $arg_list) (local.get $index))))
                (then
                  ;; If the argument is not a string, return an error
                  (return
                    (call $Stdlib_CollectString::impl::default (local.get $arg_list) (local.get $state))))
                (else
                  ;; Otherwise try the next argument until there are no remaining arguments
                  (br_if $LOOP
                    (i32.lt_u
                      (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                      (local.get $num_items))))))))
        ;; Combine the arguments into a string
        (call $Term::String::traits::collect (local.get $arg_list) (local.get $state))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectString::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $list i32)
        (local $dependencies i32)
        ;; Collect the arguments into a temporary list instance
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (local.tee $list)
        ;; Invoke the list implementation
        (call $Stdlib_CollectString::impl::List (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))
        ;; Dispose the temporary list instance
        (call $Term::drop (local.get $list))))

    (@default
      (func $Stdlib_CollectString::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectString)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
