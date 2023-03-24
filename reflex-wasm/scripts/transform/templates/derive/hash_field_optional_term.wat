;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
  (func (@concat "$" (@get $type_name) "::hash::" (@get $field_name)) (param $self i32) (param $state i64) (result i64)
    (if (result i64)
      (i32.eq (global.get $NULL) (local.tee $self (i32.load (local.get $self))))
      (then
        (call $Hash::write_byte (local.get $state) (i32.const 0)))
      (else
        (call $Hash::write_i64 (local.get $state) (call $Term::get_hash (local.get $self))))))
