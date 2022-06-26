;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $IntegersIterator
    (@struct $IntegersIterator)

    (@derive $size (@get $IntegersIterator))
    (@derive $equals (@get $IntegersIterator))
    (@derive $hash (@get $IntegersIterator))

    (@export $IntegersIterator (@get $IntegersIterator)))

  (export "isIntegersIterator" (func $Term::IntegersIterator::is))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Term::IntegersIterator::INSTANCE (mut i32) (i32.const -1))

  (func $Term::IntegersIterator::startup
    ;; Pre-allocate the singleton instances
    (global.set $Term::IntegersIterator::INSTANCE (call $Term::TermType::IntegersIterator::new)))

  (func $Term::IntegersIterator::new (export "createIntegersIterator") (result i32)
    (global.get $Term::IntegersIterator::INSTANCE))

  (func $Term::IntegersIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::IntegersIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::IntegersIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::IntegersIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::IntegersIterator::traits::size_hint (param $self i32) (result i32)
    (global.get $NULL))

  (func $Term::IntegersIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Term::Int::new
      (local.tee $iterator_state
        (select
          (i32.const 0)
          (i32.add (local.get $iterator_state) (i32.const 1))
          (i32.eq (global.get $NULL) (local.get $iterator_state)))))
    (local.get $iterator_state)
    (global.get $NULL)))
