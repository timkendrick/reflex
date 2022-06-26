;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Lambda
    (@struct $Lambda
      (@field $num_args i32)
      (@field $body (@ref $Term)))

    (@derive $size (@get $Lambda))
    (@derive $equals (@get $Lambda))
    (@derive $hash (@get $Lambda))

    (@export $Lambda (@get $Lambda)))

  (export "isLambda" (func $Term::Lambda::is))
  (export "getLambdaNumArgs" (func $Term::Lambda::get::num_args))
  (export "getLambdaBody" (func $Term::Lambda::get::body))

  (func $Term::Lambda::new (export "createLambda") (param $num_args i32) (param $body i32) (result i32)
    (call $Term::TermType::Lambda::new (local.get $num_args) (local.get $body)))

  (func $Term::Lambda::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $Term::Lambda::get::body (local.get $self))))

  (func $Term::Lambda::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Lambda::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "(")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string (call $Term::Lambda::get::num_args (local.get $self)) (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ") => ")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Term::traits::display (call $Term::Lambda::get::body (local.get $self)) (local.get $offset)))

  (func $Term::Lambda::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $num_args i32)
    (local $substituted_body i32)
    (local.set $num_args (call $Term::Lambda::get::num_args (local.get $self)))
    (local.set $substituted_body
      (call $Term::traits::substitute
        (call $Term::Lambda::get::body (local.get $self))
        (local.get $variables)
        (i32.add (local.get $scope_offset) (local.get $num_args))))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_body))
      (then
        (global.get $NULL))
      (else
        (call $Term::Lambda::new
          (call $Term::Lambda::get::num_args (local.get $self))
          (local.get $substituted_body)))))

  (func $Term::Lambda::traits::arity (param $self i32) (result i32)
    (call $Term::Lambda::get::num_args (local.get $self)))

  (func $Term::Lambda::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (local $result i32)
    (if (result i32 i32)
      (i32.eq
        (local.tee $result
          (if (result i32)
            ;; TODO: consider alternate substitution method for offseting variable scope rather than overloading
            (i32.eq (global.get $NULL) (local.get $args))
            (then
              (call $Term::traits::substitute
                (call $Term::Lambda::get::body (local.get $self))
                (local.get $args)
                (i32.const 0)))
            (else
              (if (result i32)
                (call $Term::List::get_length (local.get $args))
                (then
                  (call $Term::traits::substitute
                    (call $Term::Lambda::get::body (local.get $self))
                    (local.get $args)
                    (i32.const 0)))
                (else
                  (global.get $NULL))))))
        (global.get $NULL))
      (then
        (call $Term::Lambda::get::body (local.get $self))
        (global.get $NULL))
      (else
        (local.get $result)
        (global.get $NULL)))))
