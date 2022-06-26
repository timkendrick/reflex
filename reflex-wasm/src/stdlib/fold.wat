;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Fold
    (@args (@strict $self) (@strict $iteratee) (@lazy $seed))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::apply)
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_Fold::impl::<iterate>::<apply>::any (param $self i32) (param $iteratee i32) (param $seed i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (local $iterator_state i32)
        (local $dependencies i32)
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (loop $LOOP
          (call $Term::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          (if
            (i32.eq (local.tee $value) (global.get $NULL))
            (then)
            (else
              (call $Term::traits::apply
                (local.get $iteratee)
                (call $List::create_pair (local.get $seed) (local.get $value))
                (local.get $state))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (call $Term::traits::evaluate (local.get $state))
              (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
              (local.set $seed)
              (br $LOOP))))
        (local.get $seed)
        (local.get $dependencies)))

    (@default
      (func $Stdlib_Fold::impl::default (param $self i32) (param $iteratee i32) (param $seed i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Fold)
            (call $List::create_triple (local.get $self) (local.get $iteratee) (local.get $seed))))
        (global.get $NULL)))))
