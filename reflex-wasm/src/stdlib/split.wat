;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Split
    (@args (@strict $self) (@strict $pattern))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Split::impl::String::String (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (call $Term::String::split
          (local.get $self)
          (call $Term::String::get_offset (local.get $pattern))
          (call $Term::String::get_length (local.get $pattern)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Split::impl::default (param $self i32) (param $pattern i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Split)
            (call $Term::List::create_pair (local.get $self) (local.get $pattern))))
        (global.get $NULL)))))
