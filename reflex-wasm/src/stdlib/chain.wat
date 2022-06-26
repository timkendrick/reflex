;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Chain
    (@args (@strict $self) (@strict $other))

    (@impl
      (call $Term::implements::iterate)
      (call $Term::implements::iterate)
      (func $Stdlib_Chain::impl::<iterate>::<iterate> (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::ChainIterator::create_pair (local.get $self) (local.get $other))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Chain::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Chain)
            (call $Term::List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
