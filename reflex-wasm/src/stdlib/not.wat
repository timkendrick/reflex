;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Not "Not"
    (@args (@strict $self))

    (@default
      (func $Stdlib_Not::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (i32.eqz (call $Term::traits::is_truthy (local.get $self))))
        (global.get $NULL)))))
