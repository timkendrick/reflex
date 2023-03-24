;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $union_name) "::traits::equals") (param $self i32) (param $other i32) (result i32)
    (local $type i32)
    ;; If the two discriminants are not equal, return false
    (if (result i32)
      (i32.ne
        (local.tee $type (i32.load offset=0 (local.get $self)))
        (i32.load (local.get $other)))
      (then
        (i32.const 0))
      (else
        ;; Otherwise advance the pointer to the variant contents
        (local.set $self (i32.add (local.get $self) (i32.const 4)))
        (local.set $other (i32.add (local.get $other) (i32.const 4)))
        ;; Invoke the underlying variant equality implementation
        (@branch
          (local.get $type)
          (@list
            (@map $variant
              (@get $union_variants)
              (@block
                (return (call (@concat "$" (@get $variant) "::traits::equals") (local.get $self) (local.get $other))))))
          (i32.const 0)))))
