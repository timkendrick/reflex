;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Imported Date functions
  (func $Utils::Date::parse (import "Date" "parse") (param i32 i32) (result i64))
  (func $Utils::Date::to_json (import "Date" "toJson") (param i64 i32) (result i32)))
