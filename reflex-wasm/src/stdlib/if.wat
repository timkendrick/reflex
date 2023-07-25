;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_If "If"
    (@args (@strict $self) (@strict $consequent) (@strict $alternate))

    (@default
      (func $Stdlib_If::impl::default (param $self i32) (param $consequent i32) (param $alternate i32) (param $state i32) (result i32 i32)
        ;; Select the consequent or alternate branch, depending on whether the condition is truthy
        (select
          (local.get $consequent)
          (local.get $alternate)
          (call $Term::traits::is_truthy (local.get $self)))
        ;; Invoke the branch function with an empty argument list
        (call $Term::traits::apply (call $Term::List::empty) (local.get $state))))))
