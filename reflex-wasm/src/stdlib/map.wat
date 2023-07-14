;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Map "Map"
    (@args (@strict $self) (@strict $iteratee))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::apply)
      (func $Stdlib_Map::impl::<iterate>::<apply> (param $self i32) (param $iteratee i32) (param $state i32) (result i32 i32)
        (call $Term::MapIterator::new (local.get $self) (local.get $iteratee))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Map::impl::default (param $self i32) (param $iteratee i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Map)
            (call $Term::List::create_pair (local.get $self) (local.get $iteratee))))
        (global.get $NULL)))))
