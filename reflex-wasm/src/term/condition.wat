;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Condition
    (@union $Condition

      (@struct $CustomCondition
        (@field $effect_type (@ref $Term))
        (@field $payload (@ref $Term))
        (@field $token (@ref $Term)))

      (@struct $PendingCondition)

      (@struct $ErrorCondition
        (@field $payload (@ref $Term)))

      (@struct $TypeErrorCondition
        (@field $expected i32)
        (@field $received (@ref $Term)))

      (@struct $InvalidFunctionTargetCondition
        (@field $target (@ref $Term)))

      (@struct $InvalidFunctionArgsCondition
        (@field $target (@ref $Term))
        (@field $args (@ref $Term)))

      (@struct $InvalidAccessorCondition
        (@field $target (@ref $Term))
        (@field $key (@ref $Term)))

      (@struct $InvalidJsonCondition
        (@field $source (@ref $Term))
        (@field $offset i32))

      (@struct $InvalidPointerCondition))

      (@derive $size (@get $Condition))
      (@derive $equals (@get $Condition))
      (@derive $hash (@get $Condition))
      (@map $typename
        (@union_variants (@get $Condition))
        (@block
          (@derive $size (@union_variant (@get $Condition) (@get $_)))
          (@derive $equals (@union_variant (@get $Condition) (@get $_)))
          (@derive $hash (@union_variant (@get $Condition) (@get $_)))))

      (@export $Condition (@get $Condition))

      ;; Declare global term type constants
      (@map $typename
        (@union_variants (@get $Condition))
        (@block
          (global (@concat "$Condition::" (@get $typename)) (export (@concat "\"" "ConditionType_" (@get $typename) "\"")) i32 (i32.const (@get $_))))))

  (export "isCondition" (func $Term::Condition::is))
  (export "getConditionType" (func $Term::Condition::get::type))

  (@const $Term::Condition::PENDING i32 (call $Term::TermType::Condition::PendingCondition::new))
  (@const $Term::Condition::INVALID_POINTER i32 (call $Term::TermType::Condition::InvalidPointerCondition::new))

  ;; TODO: Codegen union variant constructors/accessors via macro
  (func $Term::Condition::custom (export "createCustomCondition") (param $type i32) (param $payload i32) (param $token i32) (result i32)
    (call $Term::TermType::Condition::CustomCondition::new (local.get $type) (local.get $payload) (local.get $token)))

  (func $Term::Condition::pending (export "createPendingCondition") (result i32)
    (global.get $Term::Condition::PENDING))

  (func $Term::Condition::error (export "createErrorCondition") (param $payload i32) (result i32)
    (call $Term::TermType::Condition::ErrorCondition::new (local.get $payload)))

  (func $Term::Condition::type_error (export "createTypeErrorCondition") (param $type i32) (param $received i32) (result i32)
    (call $Term::TermType::Condition::TypeErrorCondition::new (local.get $type) (local.get $received)))

  (func $Term::Condition::invalid_function_target (export "createInvalidFunctionTargetCondition") (param $target i32) (result i32)
    (call $Term::TermType::Condition::InvalidFunctionTargetCondition::new (local.get $target)))

  (func $Term::Condition::invalid_function_args (export "createInvalidFunctionArgsCondition") (param $target i32) (param $args i32) (result i32)
    (call $Term::TermType::Condition::InvalidFunctionArgsCondition::new (local.get $target) (local.get $args)))

  (func $Term::Condition::invalid_accessor (export "createInvalidAccessorCondition") (param $target i32) (param $key i32) (result i32)
    (call $Term::TermType::Condition::InvalidAccessorCondition::new (local.get $target) (local.get $key)))

  (func $Term::Condition::invalid_json (export "createInvalidJsonCondition") (param $source i32) (param $offset i32) (result i32)
    (call $Term::TermType::Condition::InvalidJsonCondition::new (local.get $source) (local.get $offset)))

  (func $Term::Condition::invalid_pointer (result i32)
    (global.get $Term::Condition::INVALID_POINTER))

  (func $Term::Condition::CustomCondition::get::effect_type (export "getCustomConditionEffectType") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $CustomCondition::get::effect_type))

  (func $Term::Condition::CustomCondition::get::payload (export "getCustomConditionPayload") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $CustomCondition::get::payload))

  (func $Term::Condition::CustomCondition::get::token (export "getCustomConditionToken") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $CustomCondition::get::token))

  (func $Term::Condition::ErrorCondition::get::payload (export "getErrorConditionPayload") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $ErrorCondition::get::payload))

  (func $Term::Condition::TypeErrorCondition::get::expected (export "getTypeErrorConditionExpected") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $TypeErrorCondition::get::expected))

  (func $Term::Condition::TypeErrorCondition::get::received (export "getTypeErrorConditionReceived") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $TypeErrorCondition::get::received))

  (func $Term::Condition::InvalidFunctionTargetCondition::get::target (export "getInvalidFunctionTargetConditionTarget") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidFunctionTargetCondition::get::target))

  (func $Term::Condition::InvalidFunctionArgsCondition::get::target (export "getInvalidFunctionArgsConditionTarget") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidFunctionArgsCondition::get::target))

  (func $Term::Condition::InvalidFunctionArgsCondition::get::args (export "getInvalidFunctionArgsConditionArgs") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidFunctionArgsCondition::get::args))

  (func $Term::Condition::InvalidAccessorCondition::get::target (export "getInvalidAccessorConditionTarget") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidAccessorCondition::get::target))

  (func $Term::Condition::InvalidAccessorCondition::get::key (export "getInvalidAccessorConditionKey") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidAccessorCondition::get::key))

  (func $Term::Condition::InvalidJsonCondition::get::target (export "getInvalidJsonConditionSource") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidJsonCondition::get::source))

  (func $Term::Condition::InvalidJsonCondition::get::key (export "getInvalidJsonConditionOffset") (param $self i32) (result i32)
    (call $Term::Condition::get::value (local.get $self))
    (call $InvalidJsonCondition::get::offset))

  (func $Term::Condition::invalid_builtin_function_args (param $target i32) (param $args i32) (result i32)
    (call $Term::Condition::invalid_function_args (call $Term::Builtin::new (local.get $target)) (local.get $args)))

  (func $Term::Condition::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Condition::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Condition::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL)))
