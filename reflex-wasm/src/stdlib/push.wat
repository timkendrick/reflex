;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Push "Push"
    (@args (@strict $self) (@lazy $item))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Push::impl::List::any (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $Term::List::push (local.get $self) (local.get $item))
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Push::impl::Hashset::any (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $Term::Hashset::push (local.get $self) (local.get $item))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Push::impl::<iterate>::any (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $Term::FlattenIterator::new
          (call $Term::List::create_pair
            (local.get $self)
            (call $Term::OnceIterator::new (local.get $item))))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Push::impl::default (param $self i32) (param $item i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Push)
            (call $Term::List::create_pair (local.get $self) (local.get $item))))
        (global.get $NULL)))))
