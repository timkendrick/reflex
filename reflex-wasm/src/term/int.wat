;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Int
    (@struct $Int
      (@field $value i32))

    (@derive $size (@get $Int))
    (@derive $equals (@get $Int))
    (@derive $hash (@get $Int))

    (@export $Int (@get $Int)))

  (export "isInt" (func $Term::Int::is))
  (export "getIntValue" (func $Term::Int::get::value))

  (func $Term::Int::new (export "createInt") (param $value i32) (result i32)
    (call $Term::TermType::Int::new (local.get $value)))

  (func $Term::Int::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Int::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Int::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Int::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Put the success marker on the stack
    (global.get $TRUE)
    ;; Write the serialized value to the output string and return the updated offset
    (i32.add
      (local.get $offset)
      (call $Utils::i32::write_string
        (call $Term::Int::get::value (local.get $self))
        (local.get $offset)))))
