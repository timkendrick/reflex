;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_CollectSignal "CollectSignal"
    (@args (@variadic (@eager $arg_list)))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_CollectSignal::impl::<iterate> (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $result i32)
        (local $item i32)
        (local $iterator_state i32)
        (local $dependencies i32)
        (local.set $result (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $iterator_state (global.get $NULL))
        ;; Iterate through the arguments and combine into a single combined signal
        (@iterate $LOOP $arg_list $item $iterator_state $state $dependencies
          ;; Determine whether the argument is a signal
          (if
            (call $Term::Signal::is (local.get $item))
            (then
              ;; If the argument is a signal, add it to the accumulated combined signal
              (local.set $result (call $Term::Signal::traits::union (local.get $result) (local.get $item))))
            (else
              ;; Otherwise if the argument is not a signal, return an error
              (return
                (call $Stdlib_CollectSignal::impl::default (local.get $arg_list) (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))))
        ;; Determine whether there exists an accumulated combined signal
        (if (result i32 i32)
          (i32.eq (local.get $result) (global.get $NULL))
          (then
            ;; If there is no accumulated signal (i.e. no arguments were provided), return an error message
            (call $Stdlib_CollectSignal::impl::default (local.get $arg_list) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          (else
            ;; Otherwise if there exists an accumulated signal, return the combined signal
            (local.get $result)
            (local.get $dependencies)))))

    (@default
      (func $Stdlib_CollectSignal::impl::default (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_CollectSignal)
            (call $Term::List::of (local.get $arg_list))))
        (global.get $NULL)))))
