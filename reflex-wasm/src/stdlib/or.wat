;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Or "Or"
    (@args (@strict $self) (@lazy $other))

    (@default
      (func $Stdlib_Or::impl::default (param $self i32) (param $alternative i32) (param $state i32) (result i32 i32)
        ;; Determine whether the condition is truthy
        (if (result i32 i32)
          (call $Term::traits::is_truthy (local.get $self))
          (then
            ;; If the condition is truthy, return the condition
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise invoke the alternative function with an empty argument list
            (call $Term::traits::apply (local.get $alternative) (call $Term::List::empty) (local.get $state))))))))
