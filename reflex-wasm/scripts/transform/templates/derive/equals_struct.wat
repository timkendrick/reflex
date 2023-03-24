;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $struct_name) "::traits::equals") (param $self i32) (param $other i32) (result i32)
    (@map $field_name
      (@get $field_names)
      (@block
        (call (@concat "$" (@get $struct_name) "::equals::" (@get $field_name))
          (call (@concat "$" (@get $struct_name) "::pointer::" (@get $field_name)) (local.get $self))
          (call (@concat "$" (@get $struct_name) "::pointer::" (@get $field_name)) (local.get $other)))))
    (global.get $TRUE)
    (@map $field_name
      (@get $field_names)
      (@block
        (i32.and))))
