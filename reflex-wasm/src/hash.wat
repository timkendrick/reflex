;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; FNV-1a hash function: http://www.isthe.com/chongo/tech/comp/fnv/
  (@let $FNV_SEED (i32.const 2166136261)
    (@let $FNV_PRIME (i32.const 16777619)
      (func $Hash::new (result i32)
        (@get $FNV_SEED))

      (func $Hash::write_byte (param $self i32) (param $value i32) (result i32)
        (i32.mul (@get $FNV_PRIME) (i32.xor (local.get $self) (local.get $value))))

      (func $Hash::write_bytes (param $self i32) (param $offset i32) (param $length i32) (result i32)
        (local $index i32)
        (if (result i32)
          (i32.eq (local.get $length) (i32.const 0))
          (then
            (local.get $self))
          (else
            (loop $LOOP (result i32)
              (local.set $self (call $Hash::write_byte (local.get $self) (i32.load8_u (i32.add (local.get $offset) (local.get $index)))))
              (if (result i32)
                (i32.eq (local.tee $index (i32.add (local.get $index) (i32.const 1))) (local.get $length))
                (then
                  (local.get $self))
                (else
                  (br $LOOP)))))))

      (func $Hash::write_i32 (param $self i32) (param $value i32) (result i32)
        (local.get $self)
        ;; Hash each byte in turn
        (call $Utils::i32::get_byte (local.get $value) (i32.const 0))
        (call $Hash::write_byte)
        (call $Utils::i32::get_byte (local.get $value) (i32.const 1))
        (call $Hash::write_byte)
        (call $Utils::i32::get_byte (local.get $value) (i32.const 2))
        (call $Hash::write_byte)
        (call $Utils::i32::get_byte (local.get $value) (i32.const 3))
        (call $Hash::write_byte))

      (func $Hash::write_i64 (param $self i32) (param $value i64) (result i32)
        (local.get $self)
        ;; Hash each byte in turn
        (call $Utils::i64::get_byte (local.get $value) (i32.const 0))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 1))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 2))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 3))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 4))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 5))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 6))
        (call $Hash::write_byte)
        (call $Utils::i64::get_byte (local.get $value) (i32.const 7))
        (call $Hash::write_byte))

      (func $Hash::write_f64 (param $self i32) (param $value f64) (result i32)
        (call $Hash::write_i64 (local.get $self) (i64.reinterpret_f64 (local.get $value)))))))
