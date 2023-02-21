;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ResolveArgs "ResolveArgs"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::Lambda))
      (func $Stdlib_ResolveArgs::impl::Lambda (param $self i32) (param $state i32) (result i32 i32)
        (if (result i32 i32)
          ;; If the lambda body does not contain any references to its arguments, return it as-is
          (call $Term::Lambda::traits::is_atomic (local.get $self))
          (then
            (local.get $self)
            (global.get $NULL))
          (else
            ;; Otherwise fall back to the generic implementation
            (call $Stdlib_ResolveArgs::impl::<apply> (local.get $self) (local.get $state))))))

    (@impl
      (call $TermType::implements::apply)
      (func $Stdlib_ResolveArgs::impl::<apply> (param $self i32) (param $state i32) (result i32 i32)
        (local $num_args i32)
        (local $arg_list i32)
        (local $index i32)
        ;; TODO: Support resolving arguments for functions with optional/variadic arguments
        (call $Term::traits::arity (local.get $self))
        ;; Ignore the variadic arity flag
        (drop)
        (local.set $num_args)
        (@branch
          (local.get $num_args)
          (@list
            ;; If the provided function is nullary, return it as-is
            (return
              (local.get $self)
              (global.get $NULL))
            ;; If the provided function is unary, wrap it with a lambda that resolves the argument before invoking the function
            (return
              (call $Term::Lambda::new
                (i32.const 1)
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_Sequence))
                  (call $Term::List::create_pair
                    (call $Term::Variable::new (i32.const 0))
                    (local.get $self))))
              (global.get $NULL)))
          ;; If the provided function has multiple arguments, wrap it with a lambda that resolves all the arguments before invoking the function
          ;; First create a list of all the variable expressions (to be used within the lambda function body)
          (local.set $arg_list (call $Term::List::allocate (local.get $num_args)))
          (loop $LOOP
            (call $Term::List::set_item
              (local.get $arg_list)
              (local.get $index)
              (call $Term::Variable::new
                (i32.sub
                  (i32.sub (local.get $num_args) (i32.const 1))
                  (local.get $index))))
            (br_if $LOOP
              (i32.lt_u
                (local.tee $index (i32.add (i32.const 1) (local.get $index)))
                (local.get $num_args))))
          (local.set $arg_list (call $Term::List::init (local.get $arg_list) (local.get $num_args)))
          ;; Return the wrapper lambda
          (call $Term::Lambda::new
            (local.get $num_args)
            (call $Term::Application::new
              (call $Term::Builtin::new (global.get $Stdlib_Apply))
              (call $Term::List::create_pair
                (local.get $self)
                ;; TODO: Convert argument lists to iterators for more efficient dynamic application
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_CollectList))
                  (call $Term::List::of (local.get $arg_list))))))
          (global.get $NULL))))

    (@default
      (func $Stdlib_ResolveArgs::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveArgs)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
