;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Compiled
    (@struct $Compiled
      (@field $target i32)
      (@field $num_args i32))

    (@derive $size (@get $Compiled))
    (@derive $equals (@get $Compiled))
    (@derive $hash (@get $Compiled))

    (@export $Compiled (@get $Compiled)))

  (export "isCompiled" (func $Term::Compiled::is))
  (export "getCompiledUid" (func $Term::Compiled::get::target))
  (export "getCompiledNumArgs" (func $Term::Compiled::get::num_args))

  ;; Declare type signature for dynamic compiled function calls
  (type $Compiled (func (param $args i32) (param $state i32) (result i32 i32)))

  (func $Term::Compiled::new (export "createCompiled") (param $target i32) (param $num_args i32) (result i32)
    (call $Term::TermType::Compiled::new (local.get $target) (local.get $num_args)))

  (func $Term::Compiled::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Compiled::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Compiled::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "<compiled:")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string
      (call $Term::Compiled::get::num_args (local.get $self))
      (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ":")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string
      (call $Term::Compiled::get::target (local.get $self))
      (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::Compiled::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Compiled::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Compiled::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Compiled::traits::arity (param $self i32) (result i32)
    (call $Term::Compiled::get::num_args (local.get $self)))

  (func $Term::Compiled::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Invoke the underlying function implementation
    (call_indirect (type $Compiled)
      (local.get $args)
      (local.get $state)
      (call $Term::Compiled::get::target (local.get $self)))))
