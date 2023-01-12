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
        (@field $target (@ref $Term @optional))
        (@field $args (@ref $Term)))

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
          (global (@concat "$Condition::" (@get $typename)) (export (@concat "\"" "ConditionType_" (@get $typename) "\"")) i32 (i32.const (@get $_)))

          (func (@concat "$Condition::" (@get $typename) "::sizeof") (result i32)
            (i32.add
              ;; Add 4 bytes for the discriminant
              (i32.const 4)
              ;; Add the size of the underlying condition type
              (call (@concat "$" (@get $typename) "::sizeof"))))))

      ;; Generate display formatters for condition types
      (func $ConditionType::display (param $variant i32) (param $offset i32) (result i32)
        (@branch
          (local.get $variant)
          (@list
            (@map $typename
              (@union_variants (@get $Condition))
              (return (call (@concat "$ConditionType::" (@get $typename) "::display") (local.get $offset)))))
          (local.get $offset)))

      (@map $typename
        (@union_variants (@get $Condition))
        (@block
          (func (@concat "$ConditionType::" (@get $typename) "::display") (param $offset i32) (result i32)
            (@store-bytes $offset (@to-string (@get $typename)))
            (i32.add (local.get $offset))))))

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

  (func $Term::Condition::invalid_pointer (export "createInvalidPointerCondition") (result i32)
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

  (func $Term::Condition::invalid_builtin_function_args (param $target i32) (param $args i32) (result i32)
    (call $Term::Condition::invalid_function_args (call $Term::Builtin::new (local.get $target)) (local.get $args)))

  (func $Term::Condition::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Condition::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Condition::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $type i32)
    (@store-bytes $offset "<")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset
      (call $ConditionType::display
        (local.tee $type (call $Term::Condition::get::type (local.get $self)))
        (local.get $offset)))
    (block $BLOCK
      (@switch
        (@list
          (@list
            (i32.eq (local.get $type) (global.get $Condition::CustomCondition))
            (block
              (@store-bytes $offset ":")
              (local.set $offset (i32.add (local.get $offset)))
              (local.set $offset (call $Term::Condition::CustomCondition::traits::debug (local.get $self) (local.get $offset)))
              (br $BLOCK)))
          (@list
            (i32.eq (local.get $type) (global.get $Condition::ErrorCondition))
            (block
              (@store-bytes $offset ":")
              (local.set $offset (i32.add (local.get $offset)))
              (local.set $offset (call $Term::Condition::ErrorCondition::traits::debug (local.get $self) (local.get $offset)))
              (br $BLOCK)))
          (@list
            (i32.eq (local.get $type) (global.get $Condition::TypeErrorCondition))
            (block
              (@store-bytes $offset ":")
              (local.set $offset (i32.add (local.get $offset)))
              (local.set $offset (call $Term::Condition::TypeErrorCondition::traits::debug (local.get $self) (local.get $offset)))
              (br $BLOCK)))
          (@list
            (i32.eq (local.get $type) (global.get $Condition::InvalidFunctionTargetCondition))
            (block
              (@store-bytes $offset ":")
              (local.set $offset (i32.add (local.get $offset)))
              (local.set $offset (call $Term::Condition::InvalidFunctionTargetCondition::traits::debug (local.get $self) (local.get $offset)))
              (br $BLOCK)))
          (@list
            (i32.eq (local.get $type) (global.get $Condition::InvalidFunctionArgsCondition))
            (block
              (@store-bytes $offset ":")
              (local.set $offset (i32.add (local.get $offset)))
              (local.set $offset (call $Term::Condition::InvalidFunctionArgsCondition::traits::debug (local.get $self) (local.get $offset)))
              (br $BLOCK))))))
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::Condition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Condition::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Condition::CustomCondition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Condition::CustomCondition::get::effect_type (local.get $self))
        (local.get $offset)))
    (@store-bytes $offset ":")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Condition::CustomCondition::get::payload (local.get $self))
        (local.get $offset)))
    (@store-bytes $offset ":")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Term::traits::debug
      (call $Term::Condition::CustomCondition::get::token (local.get $self))
      (local.get $offset)))

  (func $Term::Condition::ErrorCondition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::debug
      (call $Term::Condition::ErrorCondition::get::payload (local.get $self))
      (local.get $offset)))

  (func $Term::Condition::TypeErrorCondition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (local $expected i32)
    (local.set $offset
      (if (result i32)
        (i32.ne
          (local.tee $expected
            (call $Term::Condition::TypeErrorCondition::get::expected (local.get $self)))
          (global.get $NULL))
        (then
          (call $TermType::traits::display (local.get $expected) (local.get $offset)))
        (else
          (@store-bytes $offset "<unknown>")
          (i32.add (local.get $offset)))))
    (@store-bytes $offset ":")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Term::traits::debug
      (call $Term::Condition::TypeErrorCondition::get::received (local.get $self))
      (local.get $offset)))

  (func $Term::Condition::InvalidFunctionTargetCondition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::debug
      (call $Term::Condition::InvalidFunctionTargetCondition::get::target (local.get $self))
      (local.get $offset)))

  (func $Term::Condition::InvalidFunctionArgsCondition::traits::debug (param $self i32) (param $offset i32) (result i32)
    (local $args i32)
    (local $num_args i32)
    (local $index i32)
    ;; Write the function target to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Condition::InvalidFunctionArgsCondition::get::target (local.get $self))
        (local.get $offset)))
    ;; Write the opening parenthesis to the output
    (@store-bytes $offset "(")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the argument list to the output
    (local.set $args (call $Term::Condition::InvalidFunctionArgsCondition::get::args (local.get $self)))
    (if
      ;; If the argument list is empty, bail out
      (i32.eqz (local.tee $num_args (call $Term::List::get_length (local.get $args))))
      (then)
      (else
        ;; Otherwise iterate through each argument
        (loop $LOOP
          ;; If this is not the first argument, write a comma separator to the output
          (if
            (local.get $index)
            (then
              (@store-bytes $offset ", ")
              (local.set $offset (i32.add (local.get $offset)))))
          ;; Write the argument to the output
          (local.set $offset
            (call $Term::traits::debug
              (call $Term::List::get_item (local.get $args) (local.get $index))
              (local.get $offset)))
          ;; If this is not the final argument, continue with the next one
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_args))))))
    ;; Write the closing parenthesis to the output
    (@store-bytes $offset ")")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Return the updated offset
    (local.get $offset))

  (func $Term::Condition::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL)))
