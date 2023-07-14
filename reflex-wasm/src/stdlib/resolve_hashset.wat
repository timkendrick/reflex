;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveHashset "ResolveHashset"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (func $Stdlib_ResolveHashset::impl::Hashset (param $self i32) (param $state i32) (result i32 i32)
        (local $iterator i32)
        (if (result i32 i32)
          ;; If the hashset is already fully resolved, return it as-is
          (call $Term::Hashset::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve all the values and collect them into a new hashset, short-circuiting any signals
            ;; TODO: Avoid unnecessary heap allocations for intermediate values
            (local.tee $iterator
              (call $Term::EvaluateIterator::new (local.get $self)))
            (call $Term::Hashset::traits::collect_strict (local.get $state))
            ;; Dispose the temporary iterator instance
            (call $Term::drop (local.get $iterator))))))

    (@default
      (func $Stdlib_ResolveHashset::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveHashset)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
