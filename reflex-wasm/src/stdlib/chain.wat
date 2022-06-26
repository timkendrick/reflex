;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Chain
    (@args (@strict $self) (@strict $other))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::iterate)
      (func $Stdlib_Chain::impl::<iterate>::<iterate> (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $ChainIterator::create_pair (local.get $self) (local.get $other))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Chain::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Chain)
            (call $List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
