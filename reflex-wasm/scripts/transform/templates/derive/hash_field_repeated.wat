;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::hash::" (@get $field_name)) (param $self i32) (param $state i64) (result i64)
    (local $length i32)
    (local $index i32)
    ;; Hash the array length
    (call $Hash::write_i32 (local.get $state) (local.tee $length (i32.load offset=4 (local.get $self))))
    (local.set $state)
    ;; Hash the array items
    (if (result i64)
      (i32.eq (local.get $length) (i32.const 0))
      ;; If the array is empty, nothing more to do
      (then
        (local.get $state))
      (else
        ;; Otherwise hash each of the array items
        (loop $LOOP (result i64)
          (call (@concat "$" (@get $type_name) "::hash::" (@get $field_name) "::item")
            (i32.add
              (local.get $self)
              (i32.add (i32.const 8) (i32.mul (i32.const (@get $field_size)) (local.get $index))))
            (local.get $state))
          ;; Update the accumulated hash
          (local.set $state)
          ;; If this was not the final item, continue with the next item
          (br_if $LOOP (i32.ne (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length)))
          ;; Otherwise return the accumulated hash
          (local.get $state)))))
