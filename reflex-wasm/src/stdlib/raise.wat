;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Raise "Raise"
    (@args (@strict $payload))

    (@default
      (func $Stdlib_Raise::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::error
            (local.get $self)))
        (global.get $NULL)))))
