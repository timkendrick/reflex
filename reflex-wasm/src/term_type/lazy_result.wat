;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $LazyResult
    (@struct $LazyResult
      (@field $value (@ref $Term))
      (@field $dependencies (@ref $Term)))

    (@derive $size (@get $LazyResult))
    (@derive $equals (@get $LazyResult))
    (@derive $hash (@get $LazyResult))

    (@export $LazyResult (@get $LazyResult)))

  (export "isLazyResult" (func $Term::LazyResult::is))
  (export "getLazyResultValue" (func $Term::LazyResult::get::value))
  (export "getLazyResultDependencies" (func $Term::LazyResult::get::dependencies))

  (func $Term::LazyResult::new (export "createLazyResult") (param $value i32) (param $dependencies i32) (result i32)
    (call $Term::TermType::LazyResult::new (local.get $value) (local.get $dependencies)))

  (func $Term::LazyResult::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::LazyResult::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "<lazy:")
    (local.set $offset (i32.add (local.get $offset)))
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::LazyResult::get::value (local.get $self))
        (local.get $offset)))
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::LazyResult::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::LazyResult::traits::display (local.get $self) (local.get $offset)))

  (func $Term::LazyResult::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_value i32)
    (local.set $substituted_value
      (call $Term::traits::substitute
        (call $Term::LazyResult::get::value (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_value))
      (then
        (global.get $NULL))
      (else
        (call $Term::LazyResult::new
          (local.get $substituted_value)
          (call $Term::LazyResult::get::dependencies (local.get $self))))))

  (func $Term::LazyResult::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (call $Term::LazyResult::get::value (local.get $self))
    (call $Term::LazyResult::get::dependencies (local.get $self))))
