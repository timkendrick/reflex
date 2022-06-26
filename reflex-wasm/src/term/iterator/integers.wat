;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $IntegersIterator::INSTANCE (mut i32) (i32.const -1))

  (func $IntegersIterator::startup
    ;; Pre-allocate the singleton instance
    (call $Term::new (global.get $TermType::IntegersIterator) (i32.const 0))
    (call $Term::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $IntegersIterator::INSTANCE))

  (func $IntegersIterator::is (export "isIntegersIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::IntegersIterator) (call $Term::get_type (local.get $self))))

  (func $IntegersIterator::new (export "createIntegersIterator") (result i32)
    (global.get $IntegersIterator::INSTANCE))

  (func $IntegersIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $IntegersIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $IntegersIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $IntegersIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; All integers iterators are interchangeable, so no need to add anything to the hash state
    (local.get $state))

  (func $IntegersIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; All integers iterators are identical, so by definition any two integers iterators must be equal
    (global.get $TRUE))

  (func $IntegersIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $IntegersIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $IntegersIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $IntegersIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Int::new
      (local.tee $iterator_state
        (select
          (i32.const 0)
          (i32.add (local.get $iterator_state) (i32.const 1))
          (i32.eq (global.get $NULL) (local.get $iterator_state)))))
    (local.get $iterator_state)
    (global.get $NULL)))
