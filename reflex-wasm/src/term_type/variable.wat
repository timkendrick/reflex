;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Term::Variable::INSTANCE_0 i32 (call $Term::TermType::Variable::new (i32.const 0)))
  (@const $Term::Variable::INSTANCE_1 i32 (call $Term::TermType::Variable::new (i32.const 1)))
  (@const $Term::Variable::INSTANCE_2 i32 (call $Term::TermType::Variable::new (i32.const 2)))
  (@const $Term::Variable::INSTANCE_3 i32 (call $Term::TermType::Variable::new (i32.const 3)))
  (@const $Term::Variable::INSTANCE_4 i32 (call $Term::TermType::Variable::new (i32.const 4)))
  (@const $Term::Variable::INSTANCE_5 i32 (call $Term::TermType::Variable::new (i32.const 5)))
  (@const $Term::Variable::INSTANCE_6 i32 (call $Term::TermType::Variable::new (i32.const 6)))
  (@const $Term::Variable::INSTANCE_7 i32 (call $Term::TermType::Variable::new (i32.const 7)))
  (@const $Term::Variable::INSTANCE_8 i32 (call $Term::TermType::Variable::new (i32.const 8)))
  (@const $Term::Variable::INSTANCE_9 i32 (call $Term::TermType::Variable::new (i32.const 9)))
  (@const $Term::Variable::INSTANCE_10 i32 (call $Term::TermType::Variable::new (i32.const 10)))
  (@const $Term::Variable::INSTANCE_11 i32 (call $Term::TermType::Variable::new (i32.const 11)))
  (@const $Term::Variable::INSTANCE_12 i32 (call $Term::TermType::Variable::new (i32.const 12)))
  (@const $Term::Variable::INSTANCE_13 i32 (call $Term::TermType::Variable::new (i32.const 13)))
  (@const $Term::Variable::INSTANCE_14 i32 (call $Term::TermType::Variable::new (i32.const 14)))
  (@const $Term::Variable::INSTANCE_15 i32 (call $Term::TermType::Variable::new (i32.const 15)))

  (@let $Variable
    (@struct $Variable
      (@field $stack_offset i32))

    (@derive $size (@get $Variable))
    (@derive $equals (@get $Variable))
    (@derive $hash (@get $Variable))

    (@export $Variable (@get $Variable)))

  (export "isVariable" (func $Term::Variable::is))
  (export "getVariableStackOffset" (func $Term::Variable::get::stack_offset))

  (func $Term::Variable::new (export "createVariable") (param $stack_offset i32) (result i32)
    (@branch
      ;; Return a preallocated instance if one exists for the given stack offset
      (local.get $stack_offset)
      (@list
        (return (global.get $Term::Variable::INSTANCE_0))
        (return (global.get $Term::Variable::INSTANCE_1))
        (return (global.get $Term::Variable::INSTANCE_2))
        (return (global.get $Term::Variable::INSTANCE_3))
        (return (global.get $Term::Variable::INSTANCE_4))
        (return (global.get $Term::Variable::INSTANCE_5))
        (return (global.get $Term::Variable::INSTANCE_6))
        (return (global.get $Term::Variable::INSTANCE_7))
        (return (global.get $Term::Variable::INSTANCE_8))
        (return (global.get $Term::Variable::INSTANCE_9))
        (return (global.get $Term::Variable::INSTANCE_10))
        (return (global.get $Term::Variable::INSTANCE_11))
        (return (global.get $Term::Variable::INSTANCE_12))
        (return (global.get $Term::Variable::INSTANCE_13))
        (return (global.get $Term::Variable::INSTANCE_14))
        (return (global.get $Term::Variable::INSTANCE_15)))
      ;; Otherwise create a new term with the given stack offset
      (call $Term::TermType::Variable::new (local.get $stack_offset))))

  (func $Term::Variable::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Variable::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Variable::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Variable(")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string
      (call $Term::Variable::get::stack_offset (local.get $self))
      (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Variable::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Variable::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Variable::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $stack_offset i32)
    (local $num_variables i32)
    (local $replacement i32)
    (local $substituted_replacement i32)
    (local.set $stack_offset (call $Term::Variable::get::stack_offset (local.get $self)))
    ;; If this is a scope-offsetting substitution (as opposed to a scope instantiation), return the shifted variable
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $variables))
      (then
        (call $Term::Variable::new (i32.add (local.get $stack_offset) (local.get $scope_offset))))
      (else
        ;; Otherwise update the variable to reflect the scope instantiation
        ;; If the variable stack offset refers to a child scope of the scope being instantiated, it will be unaffected
        ;; by the scope instantiation, so return the unmodified marker
        (if (result i32)
          (i32.lt_u (local.get $stack_offset) (local.get $scope_offset))
          (then
            (global.get $NULL))
          (else
            ;; Otherwise if the variable stack offset refers to a parent scope of the scope being instantiated,
            ;; return a replacement variable with the adjusted stack offset (i.e. with the substituted scope removed)
            (local.set $num_variables (call $Term::List::get_length (local.get $variables)))
            (if (result i32)
              (i32.ge_u (local.get $stack_offset) (i32.add (local.get $scope_offset) (local.get $num_variables)))
              (then
                ;; Reduce the variable stack offset by the number of variables in the scope being instantiated
                (call $Term::Variable::new (i32.sub (local.get $stack_offset) (local.get $num_variables))))
              (else
                ;; Otherwise the variable stack offset refers to one of the variables in the scope being instantiated,
                ;; so get the replacement value from the instantiated scope values
                (local.set $replacement
                  (call $Term::List::get_item
                    (local.get $variables)
                    ;; Note that the order of the values provided by the scope is reversed compared to stack offsets
                    (i32.sub
                      (i32.sub (local.get $num_variables) (i32.const 1))
                      (i32.sub (local.get $stack_offset) (local.get $scope_offset)))))
                ;; Adjust any stack offsets within the replacement value to reflect the current scope traversal depth
                (local.set $substituted_replacement
                  (if (result i32)
                    (i32.eqz (local.get $scope_offset))
                    (then
                      (global.get $NULL))
                    (else
                      (call $Term::traits::substitute
                        (local.get $replacement)
                        ;; Note that providing a null variables argument indicates a scope-offsetting substitution
                        ;; (as opposed to a scope instantiation)
                        ;; TODO: consider alternatives to overloading the substitution API
                        (global.get $NULL)
                        (local.get $scope_offset)))))
                (select
                  (local.get $replacement)
                  (local.get $substituted_replacement)
                  (i32.eq (global.get $NULL) (local.get $substituted_replacement)))))))))))
