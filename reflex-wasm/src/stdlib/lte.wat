;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Lte
    (@args (@strict $self) (@strict $other))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Lte::impl::Int::Int (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Boolean::new
          (i32.le_s
            (call $Int::get::value (local.get $self))
            (call $Int::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Lte::impl::Float::Float (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Boolean::new
          (f64.le
            (call $Float::get::value (local.get $self))
            (call $Float::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Lte::impl::Int::Float (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Boolean::new
          (f64.le
            (f64.convert_i32_s (call $Int::get::value (local.get $self)))
            (call $Float::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Lte::impl::Float::Int (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Boolean::new
          (f64.le
            (call $Float::get::value (local.get $self))
            (f64.convert_i32_s (call $Int::get::value (local.get $other)))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Lte::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Lte)
            (call $List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
