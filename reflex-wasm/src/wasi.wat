;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; Imported WASI functions
  (func $WASI::fd_write (import "wasi_snapshot_preview1" "fd_write") (param i32 i32 i32 i32) (result i32)))
