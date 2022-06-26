;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Set
    (@args (@strict $self) (@strict $key) (@lazy $value))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Set::impl::List::Int::any (param $self i32) (param $key i32) (param $value i32) (param $state i32) (result i32 i32)
        (local $index i32)
        (local $length i32)
        (if (result i32 i32)
          ;; If the given key index is within the list bounds, return a new list with the updated value
          (i32.and
            (i32.ge_s (local.tee $index (call $Term::Int::get::value (local.get $key))) (i32.const 0))
            (i32.lt_u (local.get $index) (call $Term::List::get_length (local.get $self))))
          (then
            (call $Term::List::update_index (local.get $self) (local.get $index) (local.get $value))
            (global.get $NULL))
          (else
            ;; Otherwise return an error
            (call $Term::Signal::of (call $Term::Condition::invalid_accessor (local.get $self) (local.get $key)))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Set::impl::List::Float::any (param $self i32) (param $key i32) (param $value i32) (param $state i32) (result i32 i32)
        (local $index i32)
        (local $length i32)
        (if (result i32 i32)
          ;; If the given key index is within the list bounds, return a new list with the updated value
          (i32.and
            (i32.ne (local.tee $index (call $Term::Float::get_non_negative_integer_value (local.get $key))) (global.get $NULL))
            (i32.and
              (i32.ge_s (local.get $index) (i32.const 0))
              (i32.lt_u (local.get $index) (call $Term::List::get_length (local.get $self)))))
          (then
            (call $Term::List::update_index (local.get $self) (local.get $index) (local.get $value))
            (global.get $NULL))
          (else
            ;; Otherwise return an error
            (call $Term::Signal::of (call $Term::Condition::invalid_accessor (local.get $self) (local.get $key)))
            (global.get $NULL)))))


    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.or (i32.const 0xFFFFFFFF))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Set::impl::Record::any::any (param $self i32) (param $key i32) (param $value i32) (param $state i32) (result i32 i32)
        (call $Term::Record::traits::set (local.get $self) (local.get $key) (local.get $value))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (i32.or (i32.const 0xFFFFFFFF))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Set::impl::Hashmap::any::any (param $self i32) (param $key i32) (param $value i32) (param $state i32) (result i32 i32)
        (call $Term::Hashmap::traits::set (local.get $self) (local.get $key) (local.get $value))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Set::impl::default (param $self i32) (param $key i32) (param $value i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Set)
            (call $Term::List::create_triple (local.get $self) (local.get $key) (local.get $value))))
        (global.get $NULL)))))
