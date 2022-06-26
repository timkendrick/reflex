;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Builtin::startup)

  ;; Declare type signature for dynamic builtin function calls
  (type $Builtin (func (param $args i32) (param $state i32) (result i32 i32)))

  (global $ArgType::Strict i32 (i32.const 0))
  (global $ArgType::Eager i32 (i32.const 1))
  (global $ArgType::Lazy i32 (i32.const 2))

  (func $Builtin::new (export "createBuiltin") (param $target i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Builtin) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $target))
    ;; Instantiate the term
    (call $Term::init))

  (func $Builtin::is (export "isBuiltin") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Builtin) (call $Term::get_type (local.get $term))))

  (func $Builtin::get::uid (export "getBuiltinUid") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Builtin::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Builtin::traits::is_atomic (param $self i32) (result i32)
    (call $Builtin::traits::is_static (local.get $self)))

  (func $Builtin::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Builtin::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Builtin::get::uid (local.get $self))
    (call $Hash::write_i32))

  (func $Builtin::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.eq (call $Builtin::get::uid (local.get $self)) (call $Builtin::get::uid (local.get $other))))

  (func $Builtin::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Builtin::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Invoke the builtin function implementation
    (call_indirect (type $Builtin)
      (local.get $args)
      (local.get $state)
      (call $Builtin::get::uid (local.get $self)))))
