;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Slice
    (@args (@strict $self) (@strict $offset) (@strict $length))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::String::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::String::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::String::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::String::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $String::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::List::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::List::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::List::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::List::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (call $List::slice (local.get $self) (local.get $offset_value) (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::<iterate>::Int::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $SkipIterator::new (local.get $offset_value))
            (call $TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::<iterate>::Float::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $SkipIterator::new (local.get $offset_value))
            (call $TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Slice::impl::<iterate>::Int::Float (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Int::get::value (local.get $offset)))
        (local.set $length_value (call $Float::get_non_negative_integer_value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.lt_s (local.get $offset) (i32.const 0))
            (i32.eq (global.get $NULL) (local.get $length)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $SkipIterator::new (local.get $offset_value))
            (call $TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Slice::impl::<iterate>::Float::Int (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (local $offset_value i32)
        (local $length_value i32)
        (local.set $offset_value (call $Float::get_non_negative_integer_value (local.get $offset)))
        (local.set $length_value (call $Int::get::value (local.get $length)))
        (if (result i32 i32)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $offset))
            (i32.lt_s (local.get $length) (i32.const 0)))
          (then
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Slice)
                (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
            (global.get $NULL))
          (else
            (local.get $self)
            (call $SkipIterator::new (local.get $offset_value))
            (call $TakeIterator::new (local.get $length_value))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Slice::impl::default (param $self i32) (param $offset i32) (param $length i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Slice)
            (call $List::create_triple (local.get $self) (local.get $offset) (local.get $length))))
        (global.get $NULL)))))
