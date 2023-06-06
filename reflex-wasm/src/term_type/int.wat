;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Term::Int::INSTANCE_MINUS_1 i32 (call $Term::TermType::Int::new (i64.const -1)))
  (@const $Term::Int::INSTANCE_0 i32 (call $Term::TermType::Int::new (i64.const 0)))
  (@const $Term::Int::INSTANCE_1 i32 (call $Term::TermType::Int::new (i64.const 1)))
  (@const $Term::Int::INSTANCE_2 i32 (call $Term::TermType::Int::new (i64.const 2)))
  (@const $Term::Int::INSTANCE_3 i32 (call $Term::TermType::Int::new (i64.const 3)))
  (@const $Term::Int::INSTANCE_4 i32 (call $Term::TermType::Int::new (i64.const 4)))
  (@const $Term::Int::INSTANCE_5 i32 (call $Term::TermType::Int::new (i64.const 5)))
  (@const $Term::Int::INSTANCE_6 i32 (call $Term::TermType::Int::new (i64.const 6)))
  (@const $Term::Int::INSTANCE_7 i32 (call $Term::TermType::Int::new (i64.const 7)))
  (@const $Term::Int::INSTANCE_8 i32 (call $Term::TermType::Int::new (i64.const 8)))
  (@const $Term::Int::INSTANCE_9 i32 (call $Term::TermType::Int::new (i64.const 9)))

  (@let $Int
    (@struct $Int
      (@field $value i64))

    (@derive $size (@get $Int))
    (@derive $equals (@get $Int))
    (@derive $hash (@get $Int))

    (@export $Int (@get $Int)))

  (export "isInt" (func $Term::Int::is))

  (func $Term::Int::new (export "createInt") (param $value i64) (result i32)
    (@branch
      ;; Return a preallocated instance if one exists for the given value (ensuring that 64-bit overflowing integers
      ;; do not wrap around when matching the 32-bit index of the preallocated instances)
      (select
        (i32.const -1)
        ;; Preallocated instances start at -1, so we offset the value by 1 to get the index of the preallocated instance
        (i32.add (i32.wrap_i64 (local.get $value)) (i32.const 1))
        (i64.gt_u (local.get $value) (i64.const 0x00000000FFFFFFFF)))
      (@list
        (return (global.get $Term::Int::INSTANCE_MINUS_1))
        (return (global.get $Term::Int::INSTANCE_0))
        (return (global.get $Term::Int::INSTANCE_1))
        (return (global.get $Term::Int::INSTANCE_2))
        (return (global.get $Term::Int::INSTANCE_3))
        (return (global.get $Term::Int::INSTANCE_4))
        (return (global.get $Term::Int::INSTANCE_5))
        (return (global.get $Term::Int::INSTANCE_6))
        (return (global.get $Term::Int::INSTANCE_7))
        (return (global.get $Term::Int::INSTANCE_8))
        (return (global.get $Term::Int::INSTANCE_9)))
      ;; Otherwise create a new term with the given value
      (call $Term::TermType::Int::new (local.get $value))))

  (func $Term::Int::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Int::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Int::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $Utils::i64::write_string
      (call $Term::Int::get_value (local.get $self))
      (local.get $offset))
    (i32.add (local.get $offset)))

  (func $Term::Int::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Int::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Int::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Int::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Put the success marker on the stack
    (global.get $TRUE)
    ;; Write the serialized value to the output string and return the updated offset
    (i32.add
      (local.get $offset)
      (call $Utils::i64::write_string
        (call $Term::Int::get::value (local.get $self))
        (local.get $offset))))

  (func $Term::Int::get_value (export "getIntValue") (param $self i32) (result i64)
    (call $Term::Int::get::value (local.get $self))))
