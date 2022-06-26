;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Eq
    (@args (@strict $self) (@strict $payload))

    (@default
      (func $Stdlib_Eq::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Boolean::new (call $Term::traits::equals (local.get $self) (local.get $other)))
        (global.get $NULL)))))
