;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Date
    (@struct $Date
      (@field $timestamp i64))

    (@derive $size (@get $Date))
    (@derive $equals (@get $Date))
    (@derive $hash (@get $Date))

    (@export $Date (@get $Date)))

  (export "isDate" (func $Term::Date::is))
  (export "getDateTimestamp" (func $Term::Date::get::timestamp))

  (func $Term::Date::new (export "createDate") (param $timestamp i64) (result i32)
    (call $Term::TermType::Date::new (local.get $timestamp)))

  (func $Term::Date::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Date::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Date::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $Utils::Date::to_iso_string
      (call $Term::Date::get::timestamp (local.get $self))
      (local.get $offset))
    (i32.add (local.get $offset)))

  (func $Term::Date::traits::debug (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Date(")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset (call $Term::Date::traits::display (local.get $self) (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Date::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Date::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $bytes_written i32)
    ;; Write the opening quote to the output
    (@store-bytes $offset "\"")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the RFC-3339 encoded date to the output
    (local.tee $bytes_written
      (call $Utils::Date::to_iso_string
        (call $Term::Date::get::timestamp (local.get $self))
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
