;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Flatten "Flatten"
    (@args (@strict $self))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_Flatten::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::FlattenIterator::new (local.get $self))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Flatten::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Flatten)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
