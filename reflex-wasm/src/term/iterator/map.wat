;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $MapIterator::startup)

  (func $MapIterator::new (export "createMapIterator") (param $source i32) (param $iteratee i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::MapIterator) (i32.const 2)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $source))
    (call $Term::set_field (local.get $self) (i32.const 1) (local.get $iteratee))
    (call $Term::init))

  (func $MapIterator::is (export "isMapIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::MapIterator) (call $Term::get_type (local.get $self))))

  (func $MapIterator::get::source (export "getMapIteratorSource") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $MapIterator::get::iteratee (export "getMapIteratorIteratee") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 1)))

  (func $MapIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $MapIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $MapIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $MapIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $MapIterator::get::source (local.get $self))
    (call $Hash::write_term)
    (call $MapIterator::get::iteratee (local.get $self))
    (call $Hash::write_term))

  (func $MapIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (i32.and
      (call $Term::traits::equals
        (call $MapIterator::get::source (local.get $self))
        (call $MapIterator::get::source (local.get $other)))
      (call $Term::traits::equals
        (call $MapIterator::get::iteratee (local.get $self))
        (call $MapIterator::get::iteratee (local.get $other)))))

  (func $MapIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $MapIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $MapIterator::traits::size_hint (param $self i32) (result i32)
    (call $Term::traits::size_hint (call $MapIterator::get::source (local.get $self))))

  (func $MapIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Consume the next item from the source iterator
    (call $Term::traits::next (call $MapIterator::get::source (local.get $self)) (local.get $iterator_state) (local.get $state))
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
        (call $Application::new
          (call $MapIterator::get::iteratee (local.get $self))
          (call $List::of (local.get $value)))
        (local.get $iterator_state)
        (local.get $dependencies)))))
