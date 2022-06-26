;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Construct "Construct"
    (@args (@strict $self) (@strict $arg_list))

    (@impl
      (i32.eq (global.get $TermType::Constructor))
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_Construct::impl::Constructor::Record (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $instance i32)
        (if (result i32 i32)
          ;; If the record can be parsed into an instance of the constructor type, return the instance
          (i32.ne
            (local.tee $instance (call $Term::Constructor::parse_record (local.get $self) (local.get $arg_list)))
            (global.get $NULL))
          (then
            (local.get $instance)
            (global.get $NULL))
          (else
            ;; Otherwise return an error
            (call $Stdlib_Construct::impl::default (local.get $self) (local.get $arg_list) (local.get $state))))))

    (@impl
      (call $TermType::implements::apply)
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Construct::impl::<apply>::List (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::traits::apply (local.get $self) (local.get $arg_list) (local.get $state))))

    (@impl
      (call $TermType::implements::apply)
      (call $TermType::implements::iterate)
      (func $Stdlib_Construct::impl::<apply>::<iterate> (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (local.set $arg_list)
        (call $Stdlib_Construct::impl::<apply>::List (local.get $self) (local.get $arg_list) (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_Construct::impl::default (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Construct)
            (call $Term::List::create_pair (local.get $self) (local.get $arg_list))))
        (global.get $NULL)))))
