;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $EvaluateIterator
    (@struct $EvaluateIterator
      (@field $source (@ref $Term)))

    (@derive $size (@get $EvaluateIterator))
    (@derive $equals (@get $EvaluateIterator))
    (@derive $hash (@get $EvaluateIterator))

    (@export $EvaluateIterator (@get $EvaluateIterator)))

  (export "isEvaluateIterator" (func $Term::EvaluateIterator::is))
  (export "getEvaluateIteratorSource" (func $Term::EvaluateIterator::get::source))

  (func $Term::EvaluateIterator::startup)

  (func $Term::EvaluateIterator::new (export "createEvaluateIterator") (param $source i32) (result i32)
    (call $Term::TermType::EvaluateIterator::new (local.get $source)))

  (func $Term::EvaluateIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::EvaluateIterator::get::source (local.get $self)))))

  (func $Term::EvaluateIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::EvaluateIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::EvaluateIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_source))
      (then
        (global.get $NULL))
      (else
        (call $Term::EvaluateIterator::new (local.get $substituted_source)))))

  (func $Term::EvaluateIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::EvaluateIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::EvaluateIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $Term::EvaluateIterator::get::source (local.get $self))))

  (func $Term::EvaluateIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    (call $Term::traits::next
      (call $Term::EvaluateIterator::get::source (local.get $self))
      (local.get $iterator_state)
      (local.get $state))
    (local.set $dependencies)
    (local.set $iterator_state)
    (if (result i32 i32 i32)
      (i32.eq (local.tee $value) (global.get $NULL))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        (call $Term::traits::evaluate (local.get $value) (local.get $state))
        (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
        (local.get $iterator_state)
        (local.get $dependencies)))))
