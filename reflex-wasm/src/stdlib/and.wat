;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_And "And"
    (@args (@strict $self) (@strict $other))

    (@impl
      (i32.eq (global.get $TermType::Boolean))
      (i32.eq (global.get $TermType::Boolean))
      (func $Stdlib_And::impl::Boolean::Boolean (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        ;; Determine whether the condition is true
        (i32.and
          (call $Term::Boolean::get::value (local.get $self))
          (call $Term::Boolean::get::value (local.get $other)))
        ;; Construct a new boolean term with the result
        (call $Term::Boolean::new)
        (global.get $NULL)))

    (@default
      (func $Stdlib_And::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_And)
            (call $Term::List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
