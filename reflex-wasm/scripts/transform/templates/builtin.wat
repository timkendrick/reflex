;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(func (@concat "$" (@get $builtin_name) "::display") (param $offset i32) (result i32)
  (@store-bytes $offset (@get $name))
  (i32.add (local.get $offset)))

(func (@concat "$" (@get $builtin_name) "::apply") (param $args i32) (param $state i32) (result i32 i32)
  (if (result i32 i32)
    ;; If an insufficient number of arguments has been supplied, return an error signal
    (i32.lt_u (call $Term::List::get_length (local.get $args)) (i32.const (@length (@get $arg_names))))
    (then
      (call $Term::Signal::of
        (call $Term::Condition::invalid_builtin_function_args
          (global.get (@get $builtin_name))
          (local.get $args)))
      (global.get $NULL))
    (else
      ;; Extract the arguments from the argument list, pushing each argument onto the stack
      (@map $arg_name
        (@get $arg_names)
        (@block
          (call $Term::List::get_item (local.get $args) (i32.const (@get $_)))))
      (@map $arg_name
        (@get $vararg_names)
        (@block
          ;; Push the remaining variadic arguments onto the stack
          (call $Term::SkipIterator::new (local.get $args) (i32.const (@length (@get $arg_names))))))
      ;; Invoke the method implementation
      (call
        (@concat "$" (@get $builtin_name))
        (local.get $state)))))

(func (@concat "$" (@get $builtin_name) "::arity") (result i32 i32)
  (i32.const (@length (@get $arg_names)))
  (@fold $result $arg_name
    (@get $vararg_names)
    (i32.const 0)
    (i32.const 1)))

(func (@concat "$" (@get $builtin_name)) (@map $arg_name (@chain (@get $arg_names) (@get $vararg_names)) (param (@get $arg_name) i32)) (param $state i32) (result i32 i32)
  (local $dependencies i32)
  (local.set $dependencies (global.get $NULL))
  ;; Evaluate any eager arguments
  (@map $arg_name
    (@chain (@get $strict_arg_names) (@get $eager_arg_names))
    (block
      (call $Term::traits::evaluate (local.get (@get $arg_name)) (local.get $state))
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set (@get $arg_name))))
  (@map $arg_name
    (@get $eager_vararg_names)
    (block
      (call $Term::traits::size_hint (local.get (@get $arg_name)))
      (call $Term::List::collect_sized
        (call $Term::EvaluateIterator::new (local.get (@get $arg_name)))
        (local.get $state))
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set (@get $arg_name))))
  (@map $arg_name
    (@get $strict_vararg_names)
    (block
      (call $Term::traits::size_hint (local.get (@get $arg_name)))
      (call $Term::List::collect_strict_sized
        (local.get (@get $arg_name))
        (local.get $state))
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set (@get $arg_name))))
  ;; If any of the strict arguments evaluated to a signal, return a combined short-circuited signal
  (if
    (@fold $result $arg_name
      (@get $strict_arg_names)
      (i32.const 0x00000000)
      (i32.or (@get $result) (call $Term::Signal::is (local.get (@get $arg_name)))))
    (then
      (return
        (@fold $result $arg_name
          (@get $strict_arg_names)
          (global.get $NULL)
          (call $Term::Signal::traits::union
            (@get $result)
            (select
              (local.get (@get $arg_name))
              (global.get $NULL)
              (call $Term::Signal::is (local.get (@get $arg_name))))))
        (local.get $dependencies)))
    (else))
  ;; Otherwise apply the method to the evaluated arguments
  (call (@concat "$" (@get $builtin_name) "::dispatch") (@map $arg_name (@chain (@get $arg_names) (@get $vararg_names)) (local.get (@get $arg_name))) (local.get $state))
  (call $Dependencies::traits::union (local.get $dependencies)))

(func (@concat "$" (@get $builtin_name) "::dispatch") (@map $arg_name (@chain (@get $arg_names) (@get $vararg_names)) (param (@get $arg_name) i32)) (param $state i32) (result i32 i32)
  ;; Invoke the correct method implementation depending on the argument types
  (@map $arg_name
    (@chain (@get $arg_names) (@get $vararg_names))
    (local (@concat "$" (@get $arg_name) "::type") i32))
  (@map $arg_name
    (@chain (@get $arg_names) (@get $vararg_names))
    (local.set (@concat "$" (@get $arg_name) "::type") (call $Term::get_type (local.get (@get $arg_name)))))
  (@switch
    (@list
      (@zip $implementation_name $implementation_signature
        (@get $implementation_names) (@get $implementation_signatures)
        (@list
          (block (result i32)
            (@zip $arg_name $arg_signature
              (@chain (@get $arg_names) (@get $vararg_names))
              (@get $implementation_signature)
              (block (result i32)
                (local.get (@concat "$" (@get $arg_name) "::type"))
                (@get $arg_signature)))
            (@fold $result $_
              (@chain (@get $arg_names) (@get $vararg_names))
              (i32.const 0xFFFFFFFF)
              (i32.and (@get $result))))
          (return (call (@get $implementation_name) (@map $arg_name (@chain (@get $arg_names) (@get $vararg_names)) (local.get (@get $arg_name))) (local.get $state))))))
    ;; Default implementation
    (call (@get $default_implementation) (@map $arg_name (@chain (@get $arg_names) (@get $vararg_names)) (local.get (@get $arg_name))) (local.get $state))))
