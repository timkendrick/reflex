;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Nil
    (@struct $Nil)

    (@derive $size (@get $Nil))
    (@derive $equals (@get $Nil))
    (@derive $hash (@get $Nil))

    (@export $Nil (@get $Nil)))

  (export "isNil" (func $Term::Nil::is))

  (@const $Term::Nil::INSTANCE i32 (call $Term::TermType::Nil::new))

  (func $Term::Nil::new (export "createNil") (result i32)
    ;; Return the pre-allocated singleton instance
    (global.get $Term::Nil::INSTANCE))

  (func $Term::Nil::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Nil::traits::is_truthy (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Nil::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Nil::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "null")
    (i32.add (local.get $offset)))

  (func $Term::Nil::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    ;; Put the success marker on the stack
    (global.get $TRUE)
    ;; Write the serialized value to the output string and return the updated offset
    (@store-bytes $offset "null")
    (i32.add (local.get $offset))))
