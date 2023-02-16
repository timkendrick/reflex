;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $struct_name) "::traits::hash") (param $self i32) (param $state i64) (result i64)
    (@map $field_name
      (@reverse (@get $field_names))
      (call (@concat "$" (@get $struct_name) "::pointer::" (@get $field_name)) (local.get $self)))
    (local.get $state)
    (@map $field_name
      (@get $field_names)
      (call (@concat "$" (@get $struct_name) "::hash::" (@get $field_name)))))
