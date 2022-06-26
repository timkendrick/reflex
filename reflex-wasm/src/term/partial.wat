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

  (func $Term::Partial::startup)

  (func $Term::Partial::new (export "createPartial") (param $target i32) (param $args i32) (result i32)
    (call $Term::TermType::Partial::new (local.get $target) (local.get $args)))

  (func $Term::Partial::traits::is_atomic (param $self i32) (result i32)
    (i32.and
      (call $Term::traits::is_atomic (call $Term::Partial::get::target (local.get $self)))
      (call $Term::traits::is_atomic (call $Term::Partial::get::args (local.get $self)))))

  (func $Term::Partial::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

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

  (func $Term::Partial::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (call $Term::traits::apply
      (call $Term::Partial::get::target (local.get $self))
      ;; TODO: Convert argument lists to iterators for more efficient partial application
      (call $Term::List::traits::union (call $Term::Partial::get::args (local.get $self)) (local.get $args))
      (local.get $state))))
