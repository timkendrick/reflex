;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Get
    (@args (@strict $self) (@strict $key))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Get::impl::List::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.tee $value (call $Term::List::traits::get (local.get $self) (local.get $key))))
          (then
            (call $Term::Signal::of (call $Term::Condition::invalid_accessor (local.get $self) (local.get $key)))
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Get::impl::Record::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.tee $value (call $Term::Record::traits::get (local.get $self) (local.get $key))))
          (then
            (call $Term::Signal::of (call $Term::Condition::invalid_accessor (local.get $self) (local.get $key)))
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Get::impl::Hashmap::any (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.tee $value (call $Term::Hashmap::traits::get (local.get $self) (local.get $key))))
          (then
            (call $Term::Nil::new)
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Get::impl::default (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Get)
            (call $Term::List::create_pair (local.get $self) (local.get $key))))
        (global.get $NULL)))))
