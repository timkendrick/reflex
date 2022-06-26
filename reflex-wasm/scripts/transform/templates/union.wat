;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

  (func (@concat "$" (@get $union_name) "::sizeof") (result i32)
    (i32.const (@get $union_size)))

  (func (@concat "$" (@get $union_name) "::construct") (param $self i32) (param $discriminant i32)
    (@get $delegate)
    (i32.store offset=0 (local.get 0) (local.get $discriminant)))

  (func (@concat "$" (@get $union_name) "::get::type") (param $self i32) (result i32)
    (@get $delegate)
    (i32.load offset=0 (local.get 0)))

  (func (@concat "$" (@get $union_name) "::get::value") (param $self i32) (result i32)
    (@get $delegate)
    (i32.add (local.get 0) (i32.const 4)))

(@map $variant
  (@get $variants)
  (@block

  (func (@concat "$" (@get $union_name) "::" (@list_item (@get $variant) 0) "::construct") (param i32)(@map $arg (@list_item (@get $variant) 2) (@block (param (@list_item (@get $arg) 0) (@list_item (@get $arg) 1))))
    (@get $delegate)
    (call (@concat "$" (@get $union_name) "::construct") (local.get 0) (i32.const (@list_item (@get $variant) 1)))
    (call (@concat "$" (@get $union_name) "::get::value") (local.get 0))
    (call (@concat "$" (@list_item (@get $variant) 0) "::construct")(@map $arg (@list_item (@get $variant) 2) (@block (local.get (@list_item (@get $arg) 0))))))
))
