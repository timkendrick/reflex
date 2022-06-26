;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Effect "Effect"
    (@args (@strict $self) (@strict $payload) (@strict $token))

    (@default
      (func $Stdlib_Effect::impl::any::any (param $self i32) (param $payload i32) (param $token i32) (param $state i32) (result i32 i32)
        (call $Term::Effect::new
          (call $Term::Condition::custom (local.get $self) (local.get $payload) (local.get $token)))
        (global.get $NULL)))))
