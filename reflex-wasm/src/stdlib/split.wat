;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Split
    (@args (@strict $self) (@strict $pattern))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Split::impl::String::String (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (call $String::split
          (local.get $self)
          (call $String::get::offset (local.get $pattern))
          (call $String::get::length (local.get $pattern)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Split::impl::default (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Split)
            (call $List::create_pair (local.get $self) (local.get $pattern))))
        (global.get $NULL)))))
