;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Timestamp
    (@struct $Timestamp
      (@field $millis i64))

    (@derive $size (@get $Timestamp))
    (@derive $equals (@get $Timestamp))
    (@derive $hash (@get $Timestamp))

    (@export $Timestamp (@get $Timestamp)))

  (export "isTimestamp" (func $Term::Timestamp::is))
  (export "getTimestampMillis" (func $Term::Timestamp::get::millis))

  (func $Term::Timestamp::new (export "createTimestamp") (param $millis i64) (result i32)
    (call $Term::TermType::Timestamp::new (local.get $millis)))

  (func $Term::Timestamp::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Timestamp::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $Utils::Date::to_iso_string
      (call $Term::Timestamp::get::millis (local.get $self))
      (local.get $offset))
    (i32.add (local.get $offset)))

  (func $Term::Timestamp::traits::debug (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Timestamp(")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset (call $Term::Timestamp::traits::display (local.get $self) (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Timestamp::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Timestamp::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $bytes_written i32)
    ;; Write the opening quote to the output
    (@store-bytes $offset "\"")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the RFC-3339 encoded date to the output
    (local.tee $bytes_written
      (call $Utils::Date::to_iso_string
        (call $Term::Timestamp::get::millis (local.get $self))
        (local.get $offset)))
    (local.set $offset (i32.add (local.get $offset)))
    (if (result i32 i32)
      (i32.eqz (local.get $bytes_written))
      (then
        ;; Put the failure marker on the stack
        (global.get $FALSE)
        (local.get $offset))
      (else
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; Write the closing quote to the output and return the updated offset
        (@store-bytes $offset "\"")
        (i32.add (local.get $offset))))))
