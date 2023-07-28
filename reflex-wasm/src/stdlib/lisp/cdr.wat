;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Cdr "Cdr"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_Cdr::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.tee $value (call $Term::Tree::get::right (local.get $self))))
          (then
            (call $Term::Nil::new)
            (global.get $NULL))
          (else
            (local.get $value)
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Cdr::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Cdr)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
