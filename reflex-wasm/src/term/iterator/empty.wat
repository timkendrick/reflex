;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $EmptyIterator
    (@struct $EmptyIterator)

    (@derive $size (@get $EmptyIterator))
    (@derive $equals (@get $EmptyIterator))
    (@derive $hash (@get $EmptyIterator))

    (@export $EmptyIterator (@get $EmptyIterator)))

  (export "isEmptyIterator" (func $Term::EmptyIterator::is))

  (@const $Term::EmptyIterator::INSTANCE i32 (call $Term::TermType::EmptyIterator::new))

  (func $Term::EmptyIterator::new (export "createEmptyIterator") (result i32)
    (global.get $Term::EmptyIterator::INSTANCE))

  (func $Term::EmptyIterator::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::EmptyIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::EmptyIterator::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $TermType::traits::display (global.get $TermType::EmptyIterator) (local.get $offset)))

  (func $Term::EmptyIterator::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::EmptyIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::EmptyIterator::traits::size_hint (param $self i32) (result i32)
    (i32.const 0))

  (func $Term::EmptyIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    ;; Return the complete marker
    (global.get $NULL)
    (global.get $NULL)
    (global.get $NULL)))
