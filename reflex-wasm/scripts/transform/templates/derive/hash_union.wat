;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $union_name) "::traits::hash") (param $self i32) (param $state i32) (result i32)
    (local $type i32)
    ;; Write the discriminant to the hash
    (call $Hash::write_byte
      (local.get $state)
      (local.tee $type (i32.load offset=0 (local.get $self))))
    (local.set $state)
    ;; Advance the pointer to the variant contents
    (local.set $self (call (@concat "$" (@get $union_name) "::get::value") (local.get $self)))
    ;; Invoke the underlying variant hash implementation
    (@branch
      (local.get $type)
      (@list
        (@map $variant
          (@get $union_variants)
          (return (call (@concat "$" (@get $variant) "::traits::hash") (local.get $self) (local.get $state)))))
      (local.get $state)))
