;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Slice "Slice"
    (@args (@strict $self) (@strict $offset) (@strict $length))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::String::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::String::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::String::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::String::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::List::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::List::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::List::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::List::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $Term::List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::<iterate>::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $Term::SkipIterator::new (local.get $offset_value))
            (call $Term::TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::<iterate>::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $Term::SkipIterator::new (local.get $offset_value))
            (call $Term::TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::<iterate>::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset_value) (i32.const 0))
            (i32.eq (local.get $length_value) (i32.const -1)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $Term::SkipIterator::new (local.get $offset_value))
            (call $Term::TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::<iterate>::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $offset))))
        (local.set $length_value (i32.wrap_i64 (call $Term::Int::get::value (local.get $length))))
        (if (result i32 i32)
          (i32.or
            (i32.eq (local.get $offset_value) (i32.const -1))
            (i32.lt_s (local.get $length_value) (i32.const 0)))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $Term::SkipIterator::new (local.get $offset_value))
            (call $Term::TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Slice::impl::default (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Slice)
            (call $Term::List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
        (global.get $NULL)))))
