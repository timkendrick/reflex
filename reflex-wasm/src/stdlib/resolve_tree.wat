;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveTree "ResolveTree"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Tree))
      (func $Stdlib_ResolveTree::impl::Tree (param $self i32) (param $state i32) (result i32 i32)
        (local $left i32)
        (local $right i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the tree is already fully resolved, return it as-is
          (call $Term::Tree::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise resolve the child branches and create a new tree, short-circuiting any signals
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $left (call $Term::Tree::get::left (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $left) (local.get $state))))
            (local.set $dependencies)
            (local.set $left)
            (if (result i32 i32)
              (i32.eq (global.get $NULL) (local.tee $right (call $Term::Tree::get::right (local.get $self))))
              (then
                (global.get $NULL)
                (global.get $NULL))
              (else
                (call $Term::traits::evaluate (local.get $right) (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))
            (local.set $dependencies)
            (local.set $right)
            (if (result i32 i32)
              (i32.or
                (call $Term::Signal::is (local.get $right))
                (call $Term::Signal::is (local.get $left)))
              (then
                (call $Term::Signal::traits::union
                  (select
                    (local.get $left)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $left)))
                  (select
                    (local.get $right)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $right))))
                (local.get $dependencies))
              (else
                (call $Term::Tree::new (local.get $left) (local.get $right))
                (local.get $dependencies)))))))

    (@default
      (func $Stdlib_ResolveTree::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveTree)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
