;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::equals::" (@get $field_name)) (param $self i32) (param $other i32) (result i32)
    (@instruction (@block (@concat (@get $field_type) ".eq") (@instruction (@concat (@get $field_type) ".load") (local.get $self)) (@instruction (@concat (@get $field_type) ".load") (local.get $other)))))
