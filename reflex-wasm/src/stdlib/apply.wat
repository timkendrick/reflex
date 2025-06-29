;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Apply "Apply"
    (@args (@strict $self) (@strict $arg_list))

    (@impl
      (call $TermType::implements::apply)
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Apply::impl::<apply>::List (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        ;; Apply the target function to the argument list
        (call $Term::traits::apply (local.get $self) (local.get $arg_list) (local.get $state))))

    (@impl
      (call $TermType::implements::apply)
      (call $TermType::implements::iterate)
      (func $Stdlib_Apply::impl::<apply>::<iterate> (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $list i32)
        (local $dependencies i32)
        ;; Collect the argument list into a temporary list instance
        ;; TODO: Convert argument lists to iterators for more efficient dynamic application
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (local.set $list)
        ;; Invoke the list implementation
        (call $Stdlib_Apply::impl::<apply>::List (local.get $self) (local.get $list) (local.get $state))
        ;; Dispose the temporary argument list instance
        (call $Term::List::drop (local.get $list))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_Apply::impl::default (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Apply)
            (call $Term::List::create_pair (local.get $self) (local.get $arg_list))))
        (global.get $NULL)))))
