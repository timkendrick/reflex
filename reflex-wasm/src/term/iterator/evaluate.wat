;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $EvaluateIterator::startup)

  (func $EvaluateIterator::new (export "createEvaluateIterator") (param $source i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::EvaluateIterator) (i32.const 1)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::init))

  (func $EvaluateIterator::is (export "isEvaluateIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::EvaluateIterator) (call $Term::get_type (local.get $self))))

  (func $EvaluateIterator::get::source (export "getEvaluateIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $EvaluateIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $EvaluateIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $EvaluateIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $EvaluateIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $EvaluateIterator::get::source (local.get $self))
    (call $Hash::write_term))

  (func $EvaluateIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $EvaluateIterator::get::source (local.get $self))
      (call $EvaluateIterator::get::source (local.get $other))))

  (func $EvaluateIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $EvaluateIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $EvaluateIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $EvaluateIterator::get::source (local.get $self))))

  (func $EvaluateIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    (call $Term::traits::next
      (call $EvaluateIterator::get::source (local.get $self))
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
