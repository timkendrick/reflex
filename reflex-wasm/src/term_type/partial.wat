;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Partial
    (@struct $Partial
      (@field $target (@ref $Term))
      (@field $args (@ref $Term)))

    (@derive $size (@get $Partial))
    (@derive $equals (@get $Partial))
    (@derive $hash (@get $Partial))

    (@export $Partial (@get $Partial)))

  (export "isPartial" (func $Term::Partial::is))
  (export "getPartialTarget" (func $Term::Partial::get::target))
  (export "getPartialArgs" (func $Term::Partial::get::args))

  (func $Term::Partial::new (export "createPartial") (param $target i32) (param $args i32) (result i32)
    (call $Term::TermType::Partial::new (local.get $target) (local.get $args)))

  (func $Term::Partial::traits::is_atomic (param $self i32) (result i32)
    (i32.and
      (call $Term::traits::is_atomic (call $Term::Partial::get::target (local.get $self)))
      (call $Term::traits::is_atomic (call $Term::Partial::get::args (local.get $self)))))

  (func $Term::Partial::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Partial::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $args i32)
    (local $num_args i32)
    (local $index i32)
    ;; Write the function target to the output
    (local.set $offset
      (call $Term::traits::debug
        (call $Term::Partial::get::target (local.get $self))
        (local.get $offset)))
    ;; Write the opening parenthesis to the output
    (@store-bytes $offset ".bind(")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the argument list to the output
    (local.set $args (call $Term::Partial::get::args (local.get $self)))
    (if
      ;; If the argument list is empty, bail out
      (i32.eqz (local.tee $num_args (call $Term::List::get_length (local.get $args))))
      (then)
      (else
        ;; Otherwise iterate through each argument
        (loop $LOOP
          ;; If this is not the first argument, write a comma separator to the output
          (if
            (local.get $index)
            (then
              (@store-bytes $offset ", ")
              (local.set $offset (i32.add (local.get $offset)))))
          ;; Write the argument to the output
          (local.set $offset
            (call $Term::traits::debug
              (call $Term::List::get_item (local.get $args) (local.get $index))
              (local.get $offset)))
          ;; If this is not the final argument, continue with the next one
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_args))))))
    ;; Write the closing parenthesis to the output
    (@store-bytes $offset ")")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Return the updated offset
    (local.get $offset))

  (func $Term::Partial::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Partial::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Partial::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_target i32)
    (local $substituted_args i32)
    (local.set $substituted_target
      (call $Term::traits::substitute
        (call $Term::Partial::get::target (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (local.set $substituted_args
      (call $Term::traits::substitute
        (call $Term::Partial::get::args (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_target))
        (i32.eq (global.get $NULL) (local.get $substituted_args)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Partial::new
          (select
            (call $Term::Partial::get::target (local.get $self))
            (local.get $substituted_target)
            (i32.eq (global.get $NULL) (local.get $substituted_target)))
          (select
            (call $Term::Partial::get::args (local.get $self))
            (local.get $substituted_args)
            (i32.eq (global.get $NULL) (local.get $substituted_args)))))))

  (func $Term::Partial::traits::arity (param $self i32) (result i32)
    (call $Utils::i32::saturating_sub_u
      (call $Term::traits::arity (call $Term::Partial::get::target (local.get $self)))
      (call $Term::List::get_length (call $Term::Partial::get::args (local.get $self)))))

  (func $Term::Partial::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (call $Term::traits::apply
      (call $Term::Partial::get::target (local.get $self))
      ;; TODO: Convert argument lists to iterators for more efficient partial application
      (call $Term::List::traits::union
        (call $Term::Partial::get::args (local.get $self))
        (local.get $args))
      (local.get $state))))
