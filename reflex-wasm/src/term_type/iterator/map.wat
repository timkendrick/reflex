;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $MapIterator
    (@struct $MapIterator
      (@field $source (@ref $Term))
      (@field $iteratee (@ref $Term)))

    (@derive $size (@get $MapIterator))
    (@derive $equals (@get $MapIterator))
    (@derive $hash (@get $MapIterator))

    (@export $MapIterator (@get $MapIterator)))

  (export "isMapIterator" (func $Term::MapIterator::is))
  (export "getMapIteratorSource" (func $Term::MapIterator::get::source))
  (export "getMapIteratorIteratee" (func $Term::MapIterator::get::iteratee))

  (func $Term::MapIterator::new (export "createMapIterator") (param $source i32) (param $iteratee i32) (result i32)
    (call $Term::TermType::MapIterator::new (local.get $source) (local.get $iteratee)))

  (func $Term::MapIterator::traits::is_atomic (param $self i32) (result i32)
    (i32.eqz
      (call $Term::traits::size_hint
        (call $Term::MapIterator::get::source (local.get $self)))))

  (func $Term::MapIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::MapIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::MapIterator) (local.get $offset)))

  (func $Term::MapIterator::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::MapIterator::traits::display (local.get $self) (local.get $offset)))

  (func $Term::MapIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_source i32)
    (local $substituted_iteratee i32)
    (local.set $substituted_source
      (call $Term::traits::substitute
        (call $Term::MapIterator::get::source (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_iteratee
      (call $Term::traits::substitute
        (call $Term::MapIterator::get::iteratee (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_source))
        (i32.eq (global.get $NULL) (local.get $substituted_iteratee)))
      (then
        (global.get $NULL))
      (else
        (call $Term::MapIterator::new
          (select
            (call $Term::MapIterator::get::source (local.get $self))
            (local.get $substituted_source)
            (i32.eq (global.get $NULL) (local.get $substituted_source)))
          (select
            (call $Term::MapIterator::get::iteratee (local.get $self))
            (local.get $substituted_iteratee)
            (i32.eq (global.get $NULL) (local.get $substituted_iteratee)))))))

  (func $Term::MapIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::MapIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $Term::MapIterator::get::source (local.get $self))))

  (func $Term::MapIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $Term::MapIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
    (local.set $dependencies)
    (local.set $iterator_state)
    (local.set $value)
    (if (result i32 i32 i32)
      ;; If the source iterator has been fully consumed, emit the complete marker
      (i32.eq (global.get $NULL) (local.get $value))
      (then
        (global.get $NULL)
        (global.get $NULL)
        (local.get $dependencies))
      (else
        ;; Otherwise emit the transformed value and the source iterator state
        (call $Term::Application::new
          (call $Term::MapIterator::get::iteratee (local.get $self))
          (call $Term::List::of (local.get $value)))
        (local.get $iterator_state)
        (local.get $dependencies)))))
