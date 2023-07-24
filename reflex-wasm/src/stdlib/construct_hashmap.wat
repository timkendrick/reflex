;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ConstructHashmap "ConstructHashmap"
    (@args (@strict $keys) (@strict $values))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::iterate)
      (func $Stdlib_ConstructHashmap::impl::<iterate>::<iterate> (param $keys i32) (param $values i32) (param $state i32) (result i32 i32)
        (call $Term::Hashmap::traits::collect
          (call $Term::FlattenIterator::new (call $Term::ZipIterator::new (local.get $keys) (local.get $values)))
          (local.get $state))))

    (@default
      (func $Stdlib_ConstructHashmap::impl::default (param $keys i32) (param $values i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ConstructHashmap)
            (call $Term::List::create_pair (local.get $keys) (local.get $values))))
        (global.get $NULL)))))
