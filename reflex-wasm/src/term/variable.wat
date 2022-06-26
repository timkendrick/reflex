;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Variable
    (@struct $Variable
      (@field $stack_offset i32))

    (@derive $size (@get $Variable))
    (@derive $equals (@get $Variable))
    (@derive $hash (@get $Variable))

    (@export $Variable (@get $Variable)))

  (export "isVariable" (func $Term::Variable::is))
  (export "getVariableStackOffset" (func $Term::Variable::get::stack_offset))

  (func $Term::Variable::startup)

  (func $Term::Variable::new (export "createVariable") (param $stack_offset i32) (result i32)
    (call $Term::TermType::Variable::new (local.get $stack_offset)))

  (func $Term::Variable::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Variable::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

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
                ;; Reduce the variable stack offset by the number of variables in the substituted scope
                (call $Term::Variable::new (i32.sub (local.get $scope_offset) (local.get $num_variables))))
              (else
                ;; Otherwise get the replacement value from the instantiated scope values
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
                      ;; Note that providing a null variables argument indicates a scope-offsetting substitution
                      ;; (as opposed to a scope instantiation)
                      ;; TODO: consider alternatives to overloading the substitution API
                      (call $Term::traits::substitute
                        (local.get $replacement)
                        (global.get $NULL)
                        (local.get $scope_offset)))))
                (select
                  (local.get $replacement)
                  (local.get $substituted_replacement)
                  (i32.eq (global.get $NULL) (local.get $substituted_replacement))))))))))

  (func $Term::Variable::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset))))
