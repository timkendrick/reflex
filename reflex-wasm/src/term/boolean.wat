;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Boolean
    (@struct $Boolean
      (@field $value i32))

    (@derive $size (@get $Boolean))
    (@derive $equals (@get $Boolean))
    (@derive $hash (@get $Boolean))

    (@export $Boolean (@get $Boolean)))

  (export "isBoolean" (func $Term::Boolean::is))
  (export "getBooleanValue" (func $Term::Boolean::get::value))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Term::Boolean::TRUE (mut i32) (i32.const -1))
  (global $Term::Boolean::FALSE (mut i32) (i32.const -1))

  (func $Term::Boolean::startup
    ;; Pre-allocate the singleton instances
    (global.set $Term::Boolean::TRUE (call $Term::TermType::Boolean::new (i32.const 1)))
    (global.set $Term::Boolean::FALSE (call $Term::TermType::Boolean::new (i32.const 0))))

  (func $Term::Boolean::new (export "createBoolean") (param $value i32) (result i32)
    ;; Return the pre-allocated singleton instance
    (if (result i32)
      (local.get $value)
      (then
        (call $Term::Boolean::true))
      (else
        (call $Term::Boolean::false))))

  (func $Term::Boolean::true (result i32)
    (global.get $Term::Boolean::TRUE))

  (func $Term::Boolean::false (result i32)
    (global.get $Term::Boolean::FALSE))

  (func $Term::Boolean::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Boolean::traits::is_truthy (param $self i32) (result i32)
    (call $Term::Boolean::get::value (local.get $self)))

  (func $Term::Boolean::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Boolean::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Put the success marker on the stack
    (global.get $TRUE)
    ;; Write the serialized value to the output string and return the updated offset
    (if (result i32)
      (call $Term::Boolean::get::value (local.get $self))
      (then
        (@store_bytes (local.get $offset) "true")
        (i32.add (local.get $offset)))
      (else
        (@store_bytes (local.get $offset) "false")
        (i32.add (local.get $offset))))))
