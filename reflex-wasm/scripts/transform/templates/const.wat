;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(@block
  (global (@get $identifier) (export (@concat "\"" (@get $identifier) "\"")) (mut (@get $type)) (@get $default))
  (@get $initializer))
