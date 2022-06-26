;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Identity "Identity"
    (@args (@lazy $self))

    (@default
      (func $Stdlib_Identity::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))))
