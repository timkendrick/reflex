;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Abs
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Abs::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (call $Int::new (call $Utils::i32::abs (call $Int::get::value (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Abs::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (call $Float::new (f64.abs (call $Float::get::value (local.get $self))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Abs::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Abs)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
