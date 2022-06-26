;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ParseJson "ParseJson"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ParseJson::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (local $offset i32)
        (call $Json::parse (call $Term::String::get_offset (local.get $self)) (call $Term::String::get_length (local.get $self)))
        (local.set $offset)
        (local.set $value)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.get $value))
          (then
            (call $Term::Signal::of
              (call $Term::Condition::invalid_json
                (local.get $self)
                (i32.sub (local.get $offset) (call $Term::String::get_offset (local.get $self)))))
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_ParseJson::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ParseJson)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
