;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Builtin
    (@struct $Builtin
      (@field $uid i32))

    (@derive $size (@get $Builtin))
    (@derive $equals (@get $Builtin))
    (@derive $hash (@get $Builtin))

    (@export $Builtin (@get $Builtin)))

  (export "isBuiltin" (func $Term::Builtin::is))
  (export "getBuiltinUid" (func $Term::Builtin::get::uid))

  ;; Declare type signature for dynamic builtin function calls
  (type $Builtin (func (param $args i32) (param $state i32) (result i32 i32)))

  (global $ArgType::Strict i32 (i32.const 0))
  (global $ArgType::Eager i32 (i32.const 1))
  (global $ArgType::Lazy i32 (i32.const 2))

  (func $Term::Builtin::startup)

  (func $Term::Builtin::new (export "createBuiltin") (param $uid i32) (result i32)
    (call $Term::TermType::Builtin::new (local.get $uid)))

  (func $Term::Builtin::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Builtin::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Builtin::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Builtin::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Invoke the builtin function implementation
    (call_indirect (type $Builtin)
      (local.get $args)
      (local.get $state)
      (call $Term::Builtin::get::uid (local.get $self)))))
