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
            ;; Otherwise resolve the field values and create a new record, short-circuiting any signals
            (call $Stdlib_ResolveList::impl::List
              (call $Term::Record::get::values (local.get $self))
              (local.get $state))
            (local.set $dependencies)
            (local.set $values)
            (if (result i32 i32)
              (call $Term::Signal::is (local.get $values))
              (then
                (local.get $values)
                (local.get $dependencies))
              (else
                (call $Term::Record::new
                  (call $Term::Record::get::keys (local.get $self))
                  (local.get $values))
                (local.get $dependencies)))))))

    (@default
      (func $Stdlib_ResolveRecord::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveRecord)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
