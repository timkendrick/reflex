;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_PushFront
    (@args (@strict $self) (@lazy $item))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_PushFront::impl::List::any (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $List::push_front (local.get $self) (local.get $item))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_PushFront::impl::<iterate>::any (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $ChainIterator::create_pair
          (call $OnceIterator::new (local.get $item))
          (local.get $self))
        (global.get $NULL)))

    (@default
      (func $Stdlib_PushFront::impl::default (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_PushFront)
            (call $List::create_pair (local.get $self) (local.get $item))))
        (global.get $NULL)))))
