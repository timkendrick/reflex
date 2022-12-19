;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_Scan::EFFECT_NAME_SCAN "reflex::scan")

  (@builtin $Stdlib_Scan "Scan"
    (@args (@lazy $self) (@strict $seed) (@strict $iteratee))

    (@impl
      (i32.or (i32.const 0xFFFFFFFF))
      (i32.or (i32.const 0xFFFFFFFF))
      (call $TermType::implements::apply)
      (func $Stdlib_Scan::impl::any::any::<apply> (param $self i32) (param $seed i32) (param $iteratee i32) (param $state i32) (result i32 i32)
        ;; Create an effect containing the provided arguments
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Scan::EFFECT_NAME_SCAN)
            (call $Term::List::create_triple (local.get $self) (local.get $seed) (local.get $iteratee))
            (call $Term::Nil::new)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Scan::impl::default (param $self i32) (param $seed i32) (param $iteratee i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Scan)
            (call $Term::List::create_triple (local.get $self) (local.get $seed) (local.get $iteratee))))
        (global.get $NULL)))))
