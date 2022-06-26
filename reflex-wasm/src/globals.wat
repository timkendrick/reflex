;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Alias for boolean i32 values
  (global $FALSE i32 (i32.const 0))
  (global $TRUE i32 (i32.const 1))
  ;; Sentinel value used to indicate missing values
  (global $NULL i32 (i32.const 0xFFFFFFFF)))
