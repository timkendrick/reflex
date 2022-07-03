;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Let
    (@struct $Let
      (@field $initializer (@ref $Term))
      (@field $body (@ref $Term)))

    (@derive $size (@get $Let))
    (@derive $equals (@get $Let))
    (@derive $hash (@get $Let))

    (@export $Let (@get $Let)))

  (export "isLet" (func $Term::Let::is))
  (export "getLetInitializer" (func $Term::Let::get::initializer))
  (export "getLetBody" (func $Term::Let::get::body))

  (func $Term::Let::new (export "createLet") (param $initializer i32) (param $body i32) (result i32)
    (call $Term::TermType::Let::new (local.get $initializer) (local.get $body)))

  (func $Term::Let::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Let::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Let::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $branch i32)
    ;; Write the opening brace to the output
    (@store-bytes $offset "{let ")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the initializer to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Let::get::initializer (local.get $self))
        (local.get $offset)))
    ;; Write the statement separator to the output
    (@store-bytes $offset "; ")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the body to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Let::get::body (local.get $self))
        (local.get $offset)))
    ;; Write the closing brace to the output and return the updated offset
    (@store-bytes $offset "}")
    (i32.add (local.get $offset)))

  (func $Term::Let::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Let::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Let::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_initializer i32)
    (local $substituted_body i32)
    (local.set $substituted_initializer
      (call $Term::traits::substitute
        (call $Term::Let::get::initializer (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_body
      (call $Term::traits::substitute
        (call $Term::Let::get::body (local.get $self))
        (local.get $variables)
        (i32.add (local.get $scope_offset) (i32.const 1))))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_initializer))
        (i32.eq (global.get $NULL) (local.get $substituted_body)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Let::new
          (select
            (call $Term::Let::get::initializer (local.get $self))
            (local.get $substituted_initializer)
            (i32.eq (global.get $NULL) (local.get $substituted_initializer)))
          (select
            (call $Term::Let::get::body (local.get $self))
            (local.get $substituted_body)
            (i32.eq (global.get $NULL) (local.get $substituted_body)))))))

  (func $Term::Let::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $result i32)
    ;; Substitute the variable initializer into the body, leaving the result on the stack
    (if (result i32)
      (i32.eq
        (local.tee $result
          (call $Term::traits::substitute
            (call $Term::Let::get::body (local.get $self))
            (call $Term::List::of (call $Term::Let::get::initializer (local.get $self)))
            (i32.const 0)))
        (global.get $NULL))
      (then
        (call $Term::Let::get::body (local.get $self)))
      (else
        (local.get $result)))
    ;; Evaluate the result
    (call $Term::traits::evaluate (local.get $state))))
