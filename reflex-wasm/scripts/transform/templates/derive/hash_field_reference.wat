;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::hash::" (@get $field_name)) (param $self i32) (param $state i32) (result i32)
    (call (@concat "$" (@get $target_type) "::traits::hash") (i32.load (local.get $self)) (local.get $state)))
