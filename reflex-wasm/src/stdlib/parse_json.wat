;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_ParseJson
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ParseJson::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (local $offset i32)
        (call $Json::parse (call $String::get::offset (local.get $self)) (call $String::get::length (local.get $self)))
        (local.set $offset)
        (local.set $value)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.get $value))
          (then
            (call $Signal::of
              (call $Condition::invalid_json
                (local.get $self)
                (i32.sub (local.get $offset) (call $String::get::offset (local.get $self)))))
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_ParseJson::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_ParseJson)
            (call $List::of (local.get $self))))
        (global.get $NULL)))))
