;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_Variable::EFFECT_NAME_GET "reflex::variable::get")
  (@const-string $Stdlib_Variable::EFFECT_NAME_SET "reflex::variable::set")

  (@builtin $Stdlib_Variable "Variable"
    (@args (@strict $self) (@lazy $initial_value))

    (@default
      (func $Stdlib_Variable::impl::default (param $self i32) (param $initial_value i32) (param $state i32) (result i32 i32)
        ;; Create an accessor/setter pair
        (call $Term::List::create_pair
          ;; Create the accessor (see Getter implementation)
          (call $Term::Effect::new
            (call $Term::Condition::custom
              (global.get $Stdlib_Variable::EFFECT_NAME_GET)
              (call $Term::List::create_pair (local.get $self) (local.get $initial_value))
              (call $Term::Nil::new)))
          ;; Create the setter function (see Setter implementation)
          (call $Term::Lambda::new
            (i32.const 2)
            (call $Term::Application::new
              (call $Term::Builtin::new (global.get $Stdlib_Effect))
              (call $Term::List::create_triple
                (global.get $Stdlib_Variable::EFFECT_NAME_SET)
                (call $Term::List::create_pair (local.get $self) (call $Term::Variable::new (i32.const 1)))
                (call $Term::Variable::new (i32.const 0))))))
        (global.get $NULL))))

  (@builtin $Stdlib_Getter "Getter"
    (@args (@strict $self) (@lazy $initial_value))

    (@default
      (func $Stdlib_Getter::impl::default (param $self i32) (param $initial_value i32) (param $state i32) (result i32 i32)
        ;; Create a new effect that retrieves the value from state, or returns the fallback value if none has been set
        (call $Term::Effect::new
          (call $Term::Condition::custom
            (global.get $Stdlib_Variable::EFFECT_NAME_GET)
            (call $Term::List::create_pair (local.get $self) (local.get $initial_value))
            (call $Term::Nil::new)))
        (global.get $NULL))))

  (@builtin $Stdlib_Setter "Setter"
    (@args (@strict $self))

    (@default
      (func $Stdlib_Setter::impl::default (param $self i32) (param $state i32) (result i32 i32)
        ;; Create a function that accepts a value and a token, and returns an effect that updates the current value in state
        (call $Term::Lambda::new
          (i32.const 2)
          (call $Term::Application::new
            (call $Term::Builtin::new (global.get $Stdlib_Effect))
            (call $Term::List::create_triple
              (global.get $Stdlib_Variable::EFFECT_NAME_SET)
              (call $Term::List::create_pair (local.get $self) (call $Term::Variable::new (i32.const 1)))
              (call $Term::Variable::new (i32.const 0)))))
        (global.get $NULL)))))
