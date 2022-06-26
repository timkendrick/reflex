;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::equals::" (@get $field_name)) (param $self i32) (param $other i32) (result i32)
    (local $length i32)
    (local $index i32)
    (local $result i32)
    ;; If the two arrays are different lengths, return false
    (if (result i32)
      (i32.ne
        (local.tee $length (local.tee $length (i32.load offset=4 (local.get $self))))
        (i32.load offset=4 (local.get $other)))
      (then
        (global.get $FALSE))
      (else
        ;; If the array is empty, return true
        (if (result i32)
          (i32.eqz (local.get $length))
          (then
            (global.get $TRUE))
          (else
            ;; Otherwise test each of the array items for equality
            (loop $LOOP (result i32)
              ;; If the items are equal and we have not yet reached the end of the array, continue with the next item
              (br_if $LOOP
                (i32.and
                  (local.tee $result
                    (call (@concat "$" (@get $type_name) "::equals::" (@get $field_name) "::item")
                      (i32.add
                        (local.get $self)
                        (i32.add (i32.const 8) (i32.mul (i32.const (@get $field_size)) (local.get $index))))
                      (i32.add
                        (local.get $other)
                        (i32.add (i32.const 8) (i32.mul (i32.const (@get $field_size)) (local.get $index))))))
                  (i32.ne (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))))
              ;; Return the final result
              (local.get $result)))))))
