;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $OnceIterator
    (@struct $OnceIterator
      (@field $value (@ref $Term)))

    (@derive $size (@get $OnceIterator))
    (@derive $equals (@get $OnceIterator))
    (@derive $hash (@get $OnceIterator))

    (@export $OnceIterator (@get $OnceIterator)))

  (export "isOnceIterator" (func $Term::OnceIterator::is))
  (export "getOnceIteratorValue" (func $Term::OnceIterator::get::value))

  (func $Term::OnceIterator::startup)

  (func $Term::OnceIterator::new (export "createOnceIterator") (param $value i32) (result i32)
    (call $Term::TermType::OnceIterator::new (local.get $value)))

  (func $Term::OnceIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $Term::OnceIterator::get::value (local.get $self))))

  (func $Term::OnceIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::OnceIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_value i32)
    (local.set $substituted_value
      (call $Term::traits::substitute
        (call $Term::OnceIterator::get::value (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_value))
      (then
        (global.get $NULL))
      (else
        (call $Term::OnceIterator::new (local.get $substituted_value)))))

  (func $Term::OnceIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::OnceIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::OnceIterator::traits::size_hint (param $self i32) (result i32)
    (i32.const 1))

  (func $Term::OnceIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      (i32.eq (global.get $NULL) (local.get $iterator_state))
      (then
        (call $Term::OnceIterator::get::value (local.get $self))
        (i32.const 0)
        (global.get $NULL))
      (else
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL)))))
