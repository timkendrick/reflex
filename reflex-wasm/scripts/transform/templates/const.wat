;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(@block
  (global (@get $identifier) (mut (@get $type)) (i32.const -1))
  (func (@concat "$" (@get $identifier) "::initialize")
    (@get $initializer)
    (global.set (@get $identifier))))
