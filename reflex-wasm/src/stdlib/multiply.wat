;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Multiply "Multiply"
    (@args (@strict $self) (@strict $other))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Multiply::impl::Int::Int (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Int::new
          (i32.mul
            (call $Term::Int::get::value (local.get $self))
            (call $Term::Int::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Multiply::impl::Float::Float (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Float::new
          (f64.mul
            (call $Term::Float::get::value (local.get $self))
            (call $Term::Float::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Multiply::impl::Int::Float (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Float::new
          (f64.mul
            (f64.convert_i32_s (call $Term::Int::get::value (local.get $self)))
            (call $Term::Float::get::value (local.get $other))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Multiply::impl::Float::Int (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Float::new
          (f64.mul
            (call $Term::Float::get::value (local.get $self))
            (f64.convert_i32_s (call $Term::Int::get::value (local.get $other)))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Multiply::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Multiply)
            (call $Term::List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
