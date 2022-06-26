;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Constructor
    (@struct $Constructor
      (@field $keys (@ref $Term)))

    (@derive $size (@get $Constructor))
    (@derive $equals (@get $Constructor))
    (@derive $hash (@get $Constructor))

    (@export $Constructor (@get $Constructor)))

  (export "isConstructor" (func $Term::Constructor::is))
  (export "getConstructorKeys" (func $Term::Constructor::get::keys))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Term::Constructor::EMPTY (mut i32) (i32.const -1))

  (func $Term::Constructor::startup
    ;; Pre-allocate the singleton instances
    (global.set $Term::Constructor::EMPTY
      (call $Term::TermType::Constructor::new
        (call $Term::List::empty))))

  (func $Term::Constructor::new (export "createConstructor") (param $keys i32) (result i32)
    (local $self i32)
    (if (result i32)
      (i32.eq (call $Term::List::traits::length (local.get $keys)) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::Constructor::EMPTY))
      (else
        (call $Term::TermType::Constructor::new
          (local.get $keys)))))

  (func $Term::Constructor::empty (result i32)
    (global.get $Term::Constructor::EMPTY))

  (func $Term::Constructor::traits::is_atomic (param $self i32) (result i32)
    (call $Term::List::traits::is_atomic (call $Term::Constructor::get::keys (local.get $self))))

  (func $Term::Constructor::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Constructor::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_keys i32)
    (local.set $substituted_keys
      (call $Term::traits::substitute
        (call $Term::Constructor::get::keys (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_keys))
      (then
        (global.get $NULL))
      (else
        (call $Term::Constructor::new
          (local.get $substituted_keys)))))

  (func $Term::Constructor::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (local $keys i32)
    (if (result i32 i32)
      (i32.ne
        (call $Term::List::get_length (local.tee $keys (call $Term::Constructor::get::keys (local.get $self))))
        (call $Term::List::get_length (local.get $args)))
      (then
        (call $Term::Signal::of (call $Term::Condition::invalid_function_args (local.get $self) (local.get $args)))
        (global.get $NULL))
      (else
        (call $Term::Record::new (local.get $keys) (local.get $args))
        (global.get $NULL)))))
