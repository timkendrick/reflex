;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::hash::" (@get $field_name)) (param $self i32) (param $state i64) (result i64)
    (call (@concat "$" (@get $target_type) "::traits::hash") (local.get $self) (local.get $state)))
