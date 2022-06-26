;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_If
    (@args (@strict $self) (@lazy $consequent) (@lazy $alternate))

    (@default
      (func $Stdlib_If::impl::default (param $self i32) (param $consequent i32) (param $alternate i32) (param $state i32) (result i32 i32)
        (select
          (local.get $consequent)
          (local.get $alternate)
          (call $Term::traits::is_truthy (local.get $self)))
        (global.get $NULL)))))
