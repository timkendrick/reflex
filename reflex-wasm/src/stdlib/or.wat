;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Or
    (@args (@strict $self) (@lazy $other))

    (@default
      (func $Stdlib_Or::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (select
          (local.get $self)
          (local.get $other)
          (call $Term::traits::is_truthy (local.get $self)))
        (global.get $NULL)))))
