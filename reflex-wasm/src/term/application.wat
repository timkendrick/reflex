;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Application
    (@struct $Application
      (@field $target (@ref $Term))
      (@field $args (@ref $Term)))

    (@derive $size (@get $Application))
    (@derive $equals (@get $Application))
    (@derive $hash (@get $Application))

    (@export $Application (@get $Application)))

  (export "isApplication" (func $Term::Application::is))
  (export "getApplicationTarget" (func $Term::Application::get::target))
  (export "getApplicationArgs" (func $Term::Application::get::args))

  (func $Term::Application::startup)

  (func $Term::Application::new (export "createApplication") (param $target i32) (param $args i32) (result i32)
    (call $Term::TermType::Application::new (local.get $target) (local.get $args)))

  (func $Term::Application::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Application::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Application::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::Application::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    ;; TODO: Cache thunk evaluation results
    ;; Evaluate the application target
    (call $Term::traits::evaluate (call $Term::Application::get::target (local.get $self)) (local.get $state))
    ;; Pop the target dependencies from the stack, leaving just the target
    (local.set $dependencies)
    ;; Apply the target to the arguments
    (call $Term::traits::apply (call $Term::Application::get::args (local.get $self)) (local.get $state))
    ;; Pop the result dependencies and combine them with the accumulated dependencies
    (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
    ;; Evaluate the result
    (local.get $state)
    (call $Term::traits::evaluate)
    ;; Pop the result dependencies and combine them with the accumulated dependencies
    (call $Dependencies::traits::union (local.get $dependencies))))
