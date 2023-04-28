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

  (func $Term::Application::new (export "createApplication") (param $target i32) (param $args i32) (result i32)
    (local $instance i32)
    (local.tee $instance (call $Term::TermType::Application::new (local.get $target) (local.get $args))))

  (func $Term::Application::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Application::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Application::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $args i32)
    (local $num_args i32)
    (local $index i32)
    ;; Write the function target to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Application::get::target (local.get $self))
        (local.get $offset)))
    ;; Write the opening parenthesis to the output
    (@store-bytes $offset "(")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the argument list to the output
    (local.set $args (call $Term::Application::get::args (local.get $self)))
    (if
      ;; If the argument list is empty, bail out
      (i32.eqz (local.tee $num_args (call $Term::List::get_length (local.get $args))))
      (then)
      (else
        ;; Otherwise iterate through each argument
        (loop $LOOP
          ;; If this is not the first argument, write a comma separator to the output
          (if
            (local.get $index)
            (then
              (@store-bytes $offset ", ")
              (local.set $offset (i32.add (local.get $offset)))))
          ;; Write the argument to the output
          (local.set $offset
            (call $Term::traits::debug
              (call $Term::List::get_item (local.get $args) (local.get $index))
              (local.get $offset)))
          ;; If this is not the final argument, continue with the next one
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_args))))))
    ;; Write the closing parenthesis to the output
    (@store-bytes $offset ")")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Return the updated offset
    (local.get $offset))

  (func $Term::Application::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Application::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Application::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_target i32)
    (local $substituted_args i32)
    (local.set $substituted_target
      (call $Term::traits::substitute
        (call $Term::Application::get::target (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_args
      (call $Term::traits::substitute
        (call $Term::Application::get::args (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_target))
        (i32.eq (global.get $NULL) (local.get $substituted_args)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Application::new
          (select
            (call $Term::Application::get::target (local.get $self))
            (local.get $substituted_target)
            (i32.eq (global.get $NULL) (local.get $substituted_target)))
          (select
            (call $Term::Application::get::args (local.get $self))
            (local.get $substituted_args)
            (i32.eq (global.get $NULL) (local.get $substituted_args)))))))

  (func $Term::Application::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Evaluate the application target
    (call $Term::traits::evaluate (call $Term::Application::get::target (local.get $self)) (local.get $state))
    ;; Pop the target dependencies from the stack, leaving just the target
    (local.set $dependencies)
    ;; Apply the target to the arguments
    (call $Term::traits::apply (call $Term::Application::get::args (local.get $self)) (local.get $state))
    ;; Pop the result dependencies and combine them with the accumulated dependencies
    (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
    ;; Evaluate the result
    (call $Term::traits::evaluate (local.get $state))
    ;; Combine the result evaluation dependencies with the target dependencies
    (call $Dependencies::traits::union (local.get $dependencies))))
