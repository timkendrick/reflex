;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Length of a JSON-encoded ISO date string (used for determining how many bytes to allocate)
  ;; Example: "1970-01-01T00:00:00.000Z"
  (global $Term::Date::ISO_STRING_JSON_LENGTH i32 (i32.const 26))

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

  (func $Term::Date::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Date::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (local $bytes_written i32)
    (call $Allocator::extend (local.get $offset) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))
    (if (result i32 i32)
      (i32.eqz
        (local.tee $bytes_written
          (call $Utils::Date::to_json
            (call $Term::Date::get::timestamp (local.get $self))
            (local.get $offset))))
      (then
        ;; Put the failure marker on the stack
        (global.get $FALSE)
        ;; Return the updated offset
        (i32.add (local.get $offset) (global.get $Term::Date::ISO_STRING_JSON_LENGTH)))
      (else
        ;; Put the success marker on the stack
        (global.get $TRUE)
        ;; If a different number of bytes was written than expected, grow or shrink the allocated space as appropriate
        (if
          (i32.eq (local.get $bytes_written) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))
          (then)
          (else
            (if
              (i32.gt_u (local.get $bytes_written) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))
              (then
                (call $Allocator::extend
                  (i32.add (local.get $offset) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))
                  (i32.sub (local.get $bytes_written) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))))
              (else
                (call $Allocator::shrink
                  (i32.add (local.get $offset) (global.get $Term::Date::ISO_STRING_JSON_LENGTH))
                  (i32.sub (global.get $Term::Date::ISO_STRING_JSON_LENGTH) (local.get $bytes_written)))))))
        ;; Return the updated offset
        (i32.add (local.get $offset) (local.get $bytes_written))))))
