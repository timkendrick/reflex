;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Symbol
    (@struct $Symbol
      (@field $id i32))

    (@derive $size (@get $Symbol))
    (@derive $equals (@get $Symbol))
    (@derive $hash (@get $Symbol))

    (@export $Symbol (@get $Symbol)))

  (export "isSymbol" (func $Term::Symbol::is))
  (export "getSymbolId" (func $Term::Symbol::get::id))

  (func $Term::Symbol::startup)

  (func $Term::Symbol::new (export "createSymbol") (param $id i32) (result i32)
    (call $Term::TermType::Symbol::new (local.get $id)))

  (func $Term::Symbol::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Symbol::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Symbol::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL)))
