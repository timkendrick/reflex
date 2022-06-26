;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Declare condition types
  (global $ConditionType::Custom i32 (i32.const 0))
  (global $ConditionType::Pending i32 (i32.const 1))
  (global $ConditionType::Error i32 (i32.const 2))
  (global $ConditionType::TypeError i32 (i32.const 3))
  (global $ConditionType::InvalidFunctionTarget i32 (i32.const 4))
  (global $ConditionType::InvalidFunctionArgs i32 (i32.const 5))
  (global $ConditionType::InvalidAccessor i32 (i32.const 6))
  (global $ConditionType::InvalidJson i32 (i32.const 7))
  (global $ConditionType::InvalidPointer i32 (i32.const 8))

  (global $Condition::NUM_HEADER_FIELDS i32 (i32.const 1))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Condition::PENDING (mut i32) (i32.const -1))
  (global $Condition::INVALID_POINTER (mut i32) (i32.const -1))

  (func $Condition::startup
    ;; Pre-allocate the singleton instances
    (global.set $Condition::PENDING (call $Condition::startup::create_pending_singleton))
    (global.set $Condition::INVALID_POINTER (call $Condition::startup::create_invalid_pointer_singleton)))

  (func $Condition::startup::create_pending_singleton (result i32)
    (call $Condition::allocate (global.get $ConditionType::Pending) (i32.const 0))
    (call $Term::init))

  (func $Condition::startup::create_invalid_pointer_singleton (result i32)
    (call $Condition::allocate (global.get $ConditionType::InvalidPointer) (i32.const 0))
    (call $Term::init))

  (func $Condition::allocate (param $type i32) (param $num_fields i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self
      (call $Term::new
        (global.get $TermType::Condition)
        (i32.add (global.get $Condition::NUM_HEADER_FIELDS) (local.get $num_fields))))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $type)))

  (func $Condition::custom (export "createCustomCondition") (param $type i32) (param $payload i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::Custom) (i32.const 2)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $type))
    (call $Condition::set_field (local.get $self) (i32.const 1) (local.get $payload))
    (call $Term::init))

  (func $Condition::pending (export "createPendingCondition") (result i32)
    (global.get $Condition::PENDING))

  (func $Condition::error (export "createErrorCondition") (param $payload i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::Error) (i32.const 1)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $payload))
    (call $Term::init))

  (func $Condition::type_error (export "createTypeErrorCondition") (param $type i32) (param $received i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::TypeError) (i32.const 2)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $type))
    (call $Condition::set_field (local.get $self) (i32.const 1) (local.get $received))
    (call $Term::init))

  (func $Condition::invalid_function_target (param $target i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::InvalidFunctionTarget) (i32.const 1)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $target))
    (call $Term::init))

  (func $Condition::invalid_function_args (param $target i32) (param $args i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::InvalidFunctionArgs) (i32.const 2)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $target))
    (call $Condition::set_field (local.get $self) (i32.const 1) (local.get $args))
    (call $Term::init))

  (func $Condition::invalid_builtin_function_args (param $target i32) (param $args i32) (result i32)
    (call $Condition::invalid_function_args (call $Builtin::new (local.get $target)) (local.get $args)))

  (func $Condition::invalid_accessor (param $target i32) (param $key i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::InvalidAccessor) (i32.const 2)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $target))
    (call $Condition::set_field (local.get $self) (i32.const 1) (local.get $key))
    (call $Term::init))

  (func $Condition::invalid_json (param $source i32) (param $offset i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Condition::allocate (global.get $ConditionType::InvalidJson) (i32.const 2)))
    (call $Condition::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Condition::set_field (local.get $self) (i32.const 1) (local.get $offset))
    (call $Term::init))

  (func $Condition::invalid_pointer (result i32)
    (global.get $Condition::INVALID_POINTER))

  (func $Condition::is (export "isCondition") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Condition) (call $Term::get_type (local.get $term))))

  (func $Condition::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Condition::traits::is_atomic (param $self i32) (result i32)
    (call $Condition::traits::is_static (local.get $self)))

  (func $Condition::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Condition::get::type (export "getConditionType") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Condition::Custom::get::effect_type (export "getCustomConditionEffectType") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::Custom::get::effect_payload (export "getCustomConditionEffectPayload") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 1)))

  (func $Condition::Error::get::payload (export "getErrorConditionPayload") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::TypeError::get::type (export "getTypeErrorConditionType") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::TypeError::get::value (export "getTypeErrorConditionValue") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 1)))

  (func $Condition::InvalidFunctionTarget::get::target (export "getInvalidFunctionTargetConditionTarget") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::InvalidFunctionArgs::get::target (export "getInvalidFunctionArgsConditionTarget") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::InvalidFunctionArgs::get::args (export "getInvalidFunctionArgsConditionArgs") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 1)))

  (func $Condition::InvalidAccessor::get::target (export "getInvalidAccessorConditionTarget") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::InvalidAccessor::get::key (export "getInvalidAccessorConditionKey") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 1)))

  (func $Condition::InvalidJson::get::source (export "getInvalidJsonConditionSource") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 0)))

  (func $Condition::InvalidJson::get::offset (export "getInvalidJsonConditionOffset") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Condition::get_field (local.get $self) (i32.const 1)))

  (func $Condition::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Condition::get::type (local.get $self))
    (local.set $state (call $Hash::write_i32))
    ;; Invoke the correct method implementation depending on the iterator type
    (@branch
      (call $Condition::get::type (local.get $self))
      (@list
        (return (call $Condition::Custom::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::Pending::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::Error::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::TypeError::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::InvalidFunctionTarget::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::InvalidFunctionArgs::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::InvalidAccessor::traits::hash (local.get $self) (local.get $state)))
        (return (call $Condition::InvalidPointer::traits::hash (local.get $self) (local.get $state))))
      ;; Default implementation
      (local.get $state)))

  (func $Condition::Custom::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::Custom::get::effect_type (local.get $self))
    (call $Hash::write_term)
    (call $Condition::Custom::get::effect_payload (local.get $self))
    (call $Hash::write_term))

  (func $Condition::Pending::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state))

  (func $Condition::Error::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::Error::get::payload (local.get $self))
    (call $Hash::write_term))

  (func $Condition::TypeError::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::InvalidFunctionArgs::get::target (local.get $self))
    (call $Hash::write_i32)
    (call $Condition::InvalidFunctionArgs::get::args (local.get $self))
    (call $Hash::write_term))

  (func $Condition::InvalidFunctionTarget::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::InvalidFunctionTarget::get::target (local.get $self))
    (call $Hash::write_term))

  (func $Condition::InvalidFunctionArgs::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::InvalidFunctionArgs::get::target (local.get $self))
    (call $Hash::write_i32)
    (call $Condition::InvalidFunctionArgs::get::args (local.get $self))
    (call $Hash::write_term))

  (func $Condition::InvalidAccessor::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $Condition::InvalidAccessor::get::target (local.get $self))
    (call $Hash::write_term)
    (call $Condition::InvalidAccessor::get::key (local.get $self))
    (call $Hash::write_term))

  (func $Condition::InvalidPointer::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state))

  (func $Condition::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (if (result i32)
      (i32.eq (call $Condition::get::type (local.get $self)) (call $Condition::get::type (local.get $other)))
      (then
        ;; Invoke the correct method implementation depending on the iterator type
        (@branch
          (call $Condition::get::type (local.get $self))
          (@list
            (return (call $Condition::Custom::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::Pending::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::Error::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::TypeError::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::InvalidFunctionTarget::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::InvalidFunctionArgs::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::InvalidAccessor::traits::equals (local.get $self) (local.get $other)))
            (return (call $Condition::InvalidPointer::traits::equals (local.get $self) (local.get $other))))
          ;; Default implementation
          (global.get $FALSE)))
      (else
        (global.get $FALSE))))

  (func $Condition::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Condition::Custom::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $Condition::Custom::get::effect_type (local.get $self))
        (call $Condition::Custom::get::effect_type (local.get $other)))
      (call $Term::traits::equals
        (call $Condition::Custom::get::effect_payload (local.get $self))
        (call $Condition::Custom::get::effect_payload (local.get $other)))))

  (func $Condition::Pending::traits::equals (param $self i32) (param $other i32) (result i32)
    (global.get $TRUE))

  (func $Condition::Error::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $Condition::Error::get::payload (local.get $self))
      (call $Condition::Error::get::payload (local.get $other))))

  (func $Condition::TypeError::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (i32.eq
        (call $Condition::TypeError::get::type (local.get $self))
        (call $Condition::TypeError::get::type (local.get $other)))
      (call $Term::traits::equals
        (call $Condition::TypeError::get::value (local.get $self))
        (call $Condition::TypeError::get::value (local.get $other)))))

  (func $Condition::InvalidFunctionTarget::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $Condition::InvalidFunctionTarget::get::target (local.get $self))
      (call $Condition::InvalidFunctionTarget::get::target (local.get $other))))

  (func $Condition::InvalidFunctionArgs::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (i32.eq
        (call $Condition::InvalidFunctionArgs::get::target (local.get $self))
        (call $Condition::InvalidFunctionArgs::get::target (local.get $other)))
      (call $Term::traits::equals
        (call $Condition::InvalidFunctionArgs::get::args (local.get $self))
        (call $Condition::InvalidFunctionArgs::get::args (local.get $other)))))

  (func $Condition::InvalidAccessor::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $Condition::InvalidAccessor::get::target (local.get $self))
        (call $Condition::InvalidAccessor::get::target (local.get $other)))
      (call $Term::traits::equals
        (call $Condition::InvalidAccessor::get::key (local.get $self))
        (call $Condition::InvalidAccessor::get::key (local.get $other)))))

  (func $Condition::InvalidPointer::traits::equals (param $self i32) (param $other i32) (result i32)
    (global.get $TRUE))

  (func $Condition::get_field_pointer (param $self i32) (param $field_index i32) (result i32)
    (call $Term::get_field_pointer
      (local.get $self)
      (i32.add (global.get $Condition::NUM_HEADER_FIELDS) (local.get $field_index))))

  (func $Condition::get_field (param $self i32) (param $field_index i32) (result i32)
    (call $Term::get_field
      (local.get $self)
      (i32.add (global.get $Condition::NUM_HEADER_FIELDS) (local.get $field_index))))

  (func $Condition::set_field (param $self i32) (param $field_index i32) (param $value i32)
    (call $Term::set_field
      (local.get $self)
      (i32.add (global.get $Condition::NUM_HEADER_FIELDS) (local.get $field_index)) (local.get $value))))
