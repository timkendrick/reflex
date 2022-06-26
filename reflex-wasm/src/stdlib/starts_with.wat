;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_StartsWith "StartsWith"
    (@args (@strict $self) (@strict $pattern))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_StartsWith::impl::String::String (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (local $self_length i32)
        (local $pattern_length i32)
        (call $Term::Boolean::new
          (if (result i32)
            (i32.lt_u
              (local.tee $self_length (call $Term::String::get_length (local.get $self)))
              (local.tee $pattern_length (call $Term::String::get_length (local.get $pattern))))
            (then
              (global.get $FALSE))
            (else
              (call $Utils::bytes::equals
                (call $Term::String::get_offset (local.get $self))
                (local.get $pattern_length)
                (call $Term::String::get_offset (local.get $pattern))
                (local.get $pattern_length)))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_StartsWith::impl::default (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_StartsWith)
            (call $Term::List::create_pair (local.get $self) (local.get $pattern))))
        (global.get $NULL)))))
