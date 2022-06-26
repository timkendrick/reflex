;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

  (func (@concat "$" (@get $struct_name) "::sizeof") (result i32)
    (i32.const (@get $struct_size)))

  (func (@concat "$" (@get $struct_name) "::construct") (param i32)(@map $field (@get $primitive_fields) (@block (param (@list_item (@get $field) 0) (@list_item (@get $field) 2))))
    (@map $field
      (@get $primitive_fields)
      (@block
    (call (@concat "$" (@get $struct_name) "::set::" (@list_item (@get $field) 0)) (local.get 0) (local.get (@list_item (@get $field) 0))))))
(@map $field
  (@get $primitive_fields)
  (@block
  (func (@concat "$" (@get $struct_name) "::pointer::" (@list_item (@get $field) 0)) (param $self i32) (result i32)
    (@get $delegate)
    (i32.add (local.get 0) (i32.const (@list_item (@get $field) 1))))

  (func (@concat "$" (@get $struct_name) "::get::" (@list_item (@get $field) 0))(@list_item (@get $field) 4)(param $self i32) (result (@list_item (@get $field) 2))
    (@get $delegate)
    (@instruction (@concat (@list_item (@get $field) 2) ".load") (@concat "offset=" (@list_item (@get $field) 1)) (local.get 0)))

  (func (@concat "$" (@get $struct_name) "::set::" (@list_item (@get $field) 0)) (param $self i32) (param $value (@list_item (@get $field) 2))
    (@get $delegate)
    (@instruction (@concat (@list_item (@get $field) 2) ".store") (@concat "offset=" (@list_item (@get $field) 1)) (local.get 0) (local.get $value)))
))
(@map $field
  (@get $inline_fields)
  (@block
  (func (@concat "$" (@get $struct_name) "::pointer::" (@list_item (@get $field) 0))(@list_item (@get $field) 4)(param $self i32) (result (@list_item (@get $field) 2))
    (@get $delegate)
    (i32.add (local.get 0) (i32.const (@list_item (@get $field) 1))))
))
(@map $field
  (@get $repeated_fields)
  (@block
  (func (@concat "$" (@get $struct_name) "::pointer::" (@list_item (@get $field) 0)) (param $self i32) (result i32)
    (@get $delegate)
    (i32.add (local.get 0) (i32.const (@list_item (@get $field) 1))))

  (func (@concat "$" (@get $struct_name) "::get::" (@list_item (@get $field) 0) "::capacity") (param $self i32) (result i32)
    (@get $delegate)
    (i32.load (@concat "offset=" (@list_item (@get $field) 1)) (local.get 0)))

  (func (@concat "$" (@get $struct_name) "::set::" (@list_item (@get $field) 0) "::capacity") (param $self i32) (param $value i32)
    (@get $delegate)
    (i32.store (@concat "offset=" (@list_item (@get $field) 1)) (local.get 0) (local.get $value)))

  (func (@concat "$" (@get $struct_name) "::get::" (@list_item (@get $field) 0) "::length") (param $self i32) (result i32)
    (@get $delegate)
    (i32.load (@concat "offset=" (@add 4 (@list_item (@get $field) 1))) (local.get 0)))

  (func (@concat "$" (@get $struct_name) "::set::" (@list_item (@get $field) 0) "::length") (param $self i32) (param $value i32)
    (@get $delegate)
    (i32.store (@concat "offset=" (@add 4 (@list_item (@get $field) 1))) (local.get 0) (local.get $value)))

  (func (@concat "$" (@get $struct_name) "::get::" (@list_item (@get $field) 0) "::pointer") (param $self i32) (param $index i32) (result i32)
    (@get $delegate)
    (i32.add
      (i32.add (local.get 0) (i32.const (@add 8 (@list_item (@get $field) 1))))
      (i32.mul (local.get $index) (i32.const (@list_item (@get $field) 3)))))

  (func (@concat "$" (@get $struct_name) "::get::" (@list_item (@get $field) 0) "::value")(@list_item (@get $field) 4)(param $self i32) (param $index i32) (result (@list_item (@get $field) 2))
    (@get $delegate)
    (@instruction
      (@concat (@list_item (@get $field) 2) ".load")
      (@concat "offset=" (@add 8 (@list_item (@get $field) 1)))
      (i32.add
        (local.get 0)
        (i32.mul (local.get $index) (i32.const (@list_item (@get $field) 3))))))

  (func (@concat "$" (@get $struct_name) "::set::" (@list_item (@get $field) 0) "::value") (param $self i32) (param $index i32) (param $value (@list_item (@get $field) 2))
    (@get $delegate)
    (@instruction
      (@concat (@list_item (@get $field) 2) ".store")
      (@concat "offset=" (@add 8 (@list_item (@get $field) 1)))
      (i32.add
        (local.get 0)
        (i32.mul (local.get $index) (i32.const (@list_item (@get $field) 3))))
      (local.get $value)))))
