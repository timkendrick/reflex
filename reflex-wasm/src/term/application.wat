;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $Application::startup)

  (func $Application::new (export "createApplication") (param $target i32) (param $args i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Application) (i32.const 2)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $target))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $args))
    ;; Instantiate the term
    (call $Term::init))

  (func $Application::is (export "isApplication") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Application) (call $Term::get_type (local.get $term))))

  (func $Application::get::target (export "getApplicationTarget") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Application::get::args (export "getApplicationArgs") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $Application::traits::is_static (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Application::traits::is_atomic (param $self i32) (result i32)
    (call $Application::traits::is_static (local.get $self)))

  (func $Application::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Application::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Application::get::target (local.get $self))
    (call $Hash::write_term)
    (call $Application::get::args (local.get $self))
    (call $Hash::write_term))

  (func $Application::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (i32.and
      (call $Term::traits::equals (call $Application::get::target (local.get $self)) (call $Application::get::target (local.get $other)))
      (call $Term::traits::equals (call $Application::get::args (local.get $self)) (call $Application::get::args (local.get $other)))))

  (func $Application::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Application::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    ;; TODO: Cache thunk evaluation results
    ;; Evaluate the application target
    (call $Term::traits::evaluate (call $Application::get::target (local.get $self)) (local.get $state))
    ;; Pop the target dependencies from the stack, leaving just the target
    (local.set $dependencies)
    ;; Apply the target to the arguments
    (call $Term::traits::apply (call $Application::get::args (local.get $self)) (local.get $state))
    ;; Pop the result dependencies and combine them with the accumulated dependencies
    (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
    ;; Evaluate the result
    (local.get $state)
    (call $Term::traits::evaluate)
    ;; Pop the result dependencies and combine them with the accumulated dependencies
    (call $Dependencies::traits::union (local.get $dependencies))))
