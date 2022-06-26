;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ParseFloat "ParseFloat"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_ParseFloat::impl::Int (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Float::new (f64.convert_i32_s (call $Term::Int::get_value (local.get $self))))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_ParseFloat::impl::Float (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@default
      (func $Stdlib_ParseFloat::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ParseFloat)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
