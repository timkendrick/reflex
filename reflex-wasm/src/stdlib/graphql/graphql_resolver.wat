;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_GraphQlResolver::QUERY "query")
  (@const-string $Stdlib_GraphQlResolver::MUTATION "mutation")
  (@const-string $Stdlib_GraphQlResolver::SUBSCRIPTION "subscription")

  (@const $Stdlib_GraphQlResolver::CONST_2 i32
    (@depends-on $Term::Variable::INSTANCE_1)
    (call $Term::Lambda::new
      (i32.const 2)
      ;; Ignore the second argument and return the first argument
      (call $Term::Variable::new (i32.const 1))))

  (@const $Stdlib_GraphQlResolver_APPLY_CONST_2 i32
    (@depends-on $Term::Variable::INSTANCE_1)
    (@depends-on $Term::List::EMPTY)
    (call $Term::Lambda::new
      (i32.const 2)
      ;; Ignore the second argument and invoke the first argument with an empty argument list
      (call $Term::Application::new
        (call $Term::Variable::new (i32.const 1))
        (call $Term::List::empty))))

  (@builtin $Stdlib_GraphQlResolver "GraphQlResolver"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_GraphQlResolver::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $has_query_root i32)
        (local $has_mutation_root i32)
        (local $has_subscription_root i32)
        ;; Determine whether the provided record has the required operation root fields
        (call $Term::Record::traits::has (local.get $self) (global.get $Stdlib_GraphQlResolver::QUERY))
        (call $Term::Record::traits::has (local.get $self) (global.get $Stdlib_GraphQlResolver::MUTATION))
        (call $Term::Record::traits::has (local.get $self) (global.get $Stdlib_GraphQlResolver::SUBSCRIPTION))
        (i32.and)
        (i32.and)
        (if (result i32 i32)
          (then
            ;; If the provided record has all the required fields, wrap the graph root within a unary lambda factory
            (call $Term::Partial::new
              (global.get $Stdlib_GraphQlResolver::CONST_2)
              (call $Term::List::of (local.get $self)))
            (global.get $NULL))
          (else
            ;; Otherwise return an error
            (call $Stdlib_GraphQlResolver::impl::default (local.get $self) (local.get $state))))))

    (@impl
      (call $TermType::implements::apply)
      (func $Stdlib_GraphQlResolver::impl::<apply> (param $self i32) (param $state i32) (result i32 i32)
        ;; TODO: Support resolver factories with optional/variadic arguments
        (local $arity i32)
        (call $Term::traits::arity (local.get $self))
        ;; Ignore the variadic arity flag
        (drop)
        (local.set $arity)
        (@branch
          (local.get $arity)
          (@list
            ;; Wrap nullary factories with a unary lambda that immediately invokes the factory with no arguments
            (return
              (call $Term::Partial::new
                (global.get $Stdlib_GraphQlResolver_APPLY_CONST_2)
                (call $Term::List::of (local.get $self)))
              (global.get $NULL))
            ;; Return unary factories unchanged
            (return
              (local.get $self)
              (global.get $NULL)))
          ;; For factories with arity 2 or more, return an error
          (call $Stdlib_GraphQlResolver::impl::default (local.get $self) (local.get $state)))))

    (@default
      (func $Stdlib_GraphQlResolver::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_GraphQlResolver)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
