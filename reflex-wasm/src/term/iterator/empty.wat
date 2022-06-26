;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $EmptyIterator::INSTANCE (mut i32) (i32.const -1))

  (func $EmptyIterator::startup
    ;; Pre-allocate the singleton instance
    (call $Term::new (global.get $TermType::EmptyIterator) (i32.const 0))
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $EmptyIterator::INSTANCE))

  (func $EmptyIterator::new (export "createEmptyIterator") (result i32)
    (global.get $EmptyIterator::INSTANCE))

  (func $EmptyIterator::is (export "isEmptyIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::EmptyIterator) (call $Term::get_type (local.get $self))))

  (func $EmptyIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $EmptyIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $EmptyIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $EmptyIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; All empty iterators are interchangeable, so no need to add anything to the hash state
    (local.get $state))

  (func $EmptyIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; All empty iterators are identical, so by definition any two empty iterators must be equal
    (global.get $TRUE))

  (func $EmptyIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $EmptyIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $EmptyIterator::traits::size_hint (param $self i32) (result i32)
    (i32.const 0))

  (func $EmptyIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (global.get $NULL)
    (global.get $NULL)
    (global.get $NULL)))
