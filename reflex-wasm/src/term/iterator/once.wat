;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (func $OnceIterator::startup)

  (func $OnceIterator::new (export "createOnceIterator") (param $value i32) (result i32)
    (local $self i32)
    (local.tee $self (call $Term::new (global.get $TermType::OnceIterator) (i32.const 1)))
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $value))
    (call $Term::init))

  (func $OnceIterator::is (export "isOnceIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::OnceIterator) (call $Term::get_type (local.get $self))))

  (func $OnceIterator::get::value (export "getOnceIteratorValue") (param $self i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $OnceIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $OnceIterator::traits::is_atomic (param $self i32) (result i32)
    (call $Term::traits::is_atomic (call $OnceIterator::get::value (local.get $self))))

  (func $OnceIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $OnceIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    (call $OnceIterator::get::value (local.get $self))
    (call $Hash::write_term))

  (func $OnceIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (call $Term::traits::equals
      (call $OnceIterator::get::value (local.get $self))
      (call $OnceIterator::get::value (local.get $other))))

  (func $OnceIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $OnceIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $OnceIterator::traits::size_hint (param $self i32) (result i32)
    (i32.const 1))

  (func $OnceIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (if (result i32 i32 i32)
      (i32.eq (global.get $NULL) (local.get $iterator_state))
      (then
        (call $OnceIterator::get::value (local.get $self))
        (i32.const 0)
        (global.get $NULL))
      (else
        (global.get $NULL)
        (global.get $NULL)
        (global.get $NULL)))))
