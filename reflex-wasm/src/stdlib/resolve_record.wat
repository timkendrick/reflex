;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveRecord "ResolveRecord"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveRecord::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $keys i32)
        (local $values i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the record is already fully resolved, return it as-is
          (call $Term::Record::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the keys and values and create a new record, short-circuiting any signals
            (call $Stdlib_ResolveList::impl::List
              (call $Term::Record::get::keys (local.get $self))
              (local.get $state))
            (local.set $dependencies)
            (local.set $keys)
            (call $Stdlib_ResolveList::impl::List
              (call $Term::Record::get::values (local.get $self))
              (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (local.set $values)
            (if (result i32 i32)
              (i32.or
                (call $Term::Signal::is (local.get $values))
                (call $Term::Signal::is (local.get $keys)))
              (then
                (call $Term::Signal::traits::union
                  (select
                    (local.get $keys)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $keys)))
                  (select
                    (local.get $values)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $values))))
                (local.get $dependencies))
              (else
                (call $Term::Record::new (local.get $keys) (local.get $values))
                (local.get $dependencies)))))))

    (@default
      (func $Stdlib_ResolveRecord::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveRecord)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
