;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Ceil
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Ceil::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Ceil::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (call $Float::new (f64.ceil (call $Float::get::value (local.get $self))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Ceil::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Ceil)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
