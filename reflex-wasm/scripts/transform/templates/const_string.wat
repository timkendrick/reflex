;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(@const (@get $identifier) i32
  (func (result i32)
    (local $offset i32)
    (local.tee $offset (call $Term::String::allocate_unsized))
    (local.set $offset (call $Term::String::get_offset (local.get $offset)))
    (@store-bytes $offset (@get $value))
    (call $Term::String::init_unsized)))
