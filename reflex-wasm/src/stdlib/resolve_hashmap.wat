;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveHashmap "ResolveHashmap"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveHashmap::impl::Hashmap (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (local $keys_iterator i32)
        (local $values_iterator i32)
        (if (result i32 i32)
          ;; If the hashmap is already fully resolved, return it as-is
          (call $Term::Hashmap::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the entries and collect them into a new hashmap, short-circuiting any signals
            ;; TODO: Avoid unnecessary heap allocations for intermediate values
            (local.tee $keys_iterator (call $Term::HashmapKeysIterator::new (local.get $self)))
            (local.tee $values_iterator (call $Term::HashmapValuesIterator::new (local.get $self)))
            (local.tee $iterator (call $Term::ZipIterator::new))
            (call $Term::Hashmap::traits::collect_strict (local.get $state))
            ;; Dispose the temporary iterator instances
            (call $Term::drop (local.get $iterator))
            (call $Term::drop (local.get $values_iterator))
            (call $Term::drop (local.get $keys_iterator))))))

    (@default
      (func $Stdlib_ResolveHashmap::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveHashmap)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
