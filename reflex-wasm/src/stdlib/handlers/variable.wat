;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_Variable::EFFECT_NAME_GET "reflex::variable::get")
  (@const-string $Stdlib_Variable::EFFECT_NAME_SET "reflex::variable::set")
  (@const-string $Stdlib_Variable::EFFECT_NAME_INCREMENT "reflex::variable::increment")
  (@const-string $Stdlib_Variable::EFFECT_NAME_DECREMENT "reflex::variable::decrement")

  (@builtin $Stdlib_GetVariable "GetVariable"
    (@args (@strict $self) (@lazy $initial_value))

    (@impl
      (i32.eq (global.get $TermType::Symbol))
      (i32.or (i32.const 0xFFFFFFFF))
      (func $Stdlib_GetVariable::impl::Symbol::any (param $self i32) (param $initial_value i32) (param $state i32) (result i32 i32)
        ;; Create a new effect that retrieves the value from state, or returns the fallback value if none has been set
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Variable::EFFECT_NAME_GET)
            (call $Term::List::create_pair (local.get $self) (local.get $initial_value))
            (call $Term::Nil::new)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_GetVariable::impl::default (param $self i32) (param $initial_value i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_GetVariable)
            (call $Term::List::create_pair (local.get $self) (local.get $initial_value))))
        (global.get $NULL))))

  (@builtin $Stdlib_SetVariable "SetVariable"
    (@args (@strict $self) (@lazy $value) (@strict $token))

    (@impl
      (i32.eq (global.get $TermType::Symbol))
      (i32.or (i32.const 0xFFFFFFFF))
      (i32.eq (global.get $TermType::Symbol))
      (func $Stdlib_SetVariable::impl::Symbol::any::Symbol (param $self i32) (param $value i32) (param $token i32) (param $state i32) (result i32 i32)
        ;; Create a new effect that updates the current value in state, and returns the updated value
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Variable::EFFECT_NAME_SET)
            (call $Term::List::create_pair (local.get $self) (local.get $value))
            (local.get $token)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_SetVariable::impl::default (param $self i32) (param $value i32) (param $token i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_SetVariable)
            (call $Term::List::create_triple (local.get $self) (local.get $value) (local.get $token))))
        (global.get $NULL))))

  (@builtin $Stdlib_IncrementVariable "IncrementVariable"
    (@args (@strict $self) (@strict $token))

    (@impl
      (i32.eq (global.get $TermType::Symbol))
      (i32.eq (global.get $TermType::Symbol))
      (func $Stdlib_IncrementVariable::impl::Symbol::Symbol (param $self i32) (param $token i32) (param $state i32) (result i32 i32)
        ;; Create a new effect that increments the current value in state, and returns the updated value
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Variable::EFFECT_NAME_INCREMENT)
            (call $Term::List::of (local.get $self))
            (local.get $token)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_IncrementVariable::impl::default (param $self i32) (param $token i32)(param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_IncrementVariable)
            (call $Term::List::create_pair (local.get $self) (local.get $token))))
        (global.get $NULL))))

  (@builtin $Stdlib_DecrementVariable "DecrementVariable"
    (@args (@strict $self) (@strict $token))

    (@impl
      (i32.eq (global.get $TermType::Symbol))
      (i32.eq (global.get $TermType::Symbol))
      (func $Stdlib_DecrementVariable::impl::Symbol::Symbol (param $self i32) (param $token i32) (param $state i32) (result i32 i32)
        ;; Create a new effect that decrements the current value in state, and returns the updated value
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Variable::EFFECT_NAME_DECREMENT)
            (call $Term::List::of (local.get $self))
            (local.get $token)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_DecrementVariable::impl::default (param $self i32) (param $token i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_DecrementVariable)
            (call $Term::List::create_pair (local.get $self) (local.get $token))))
        (global.get $NULL)))))
