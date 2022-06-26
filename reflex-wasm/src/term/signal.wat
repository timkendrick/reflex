;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Signal::PENDING (mut i32) (i32.const -1))
  (global $Signal::INVALID_POINTER (mut i32) (i32.const -1))

  (func $Signal::startup
    ;; Pre-allocate the singleton instances
    (global.set $Signal::PENDING
      (call $Signal::new (call $Tree::of (call $Condition::pending))))
    (global.set $Signal::INVALID_POINTER
      (call $Signal::new (call $Tree::of (call $Condition::invalid_pointer)))))

  (func $Signal::new (param $conditions i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Signal) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (local.get $conditions))
    ;; Instantiate the term
    (call $Term::init))

  (func $Signal::of (export "createSignal") (param $condition i32) (result i32)
    (local $self i32)
    ;; Allocate a new struct of the required size and type
    (local.tee $self (call $Term::new (global.get $TermType::Signal) (i32.const 1)))
    ;; Store the struct fields at the correct offsets
    (call $Term::set_field (local.get $self) (i32.const 0) (call $Tree::of (local.get $condition)))
    ;; Instantiate the term
    (call $Term::init))

  (func $Signal::pending (export "createPendingSignal") (result i32)
    (global.get $Signal::PENDING))

  (func $Signal::invalid_pointer (result i32)
    (global.get $Signal::INVALID_POINTER))

  (func $Signal::is (export "isSignal") (param $term i32) (result i32)
    (i32.eq (global.get $TermType::Signal) (call $Term::get_type (local.get $term))))

  (func $Signal::get::conditions (export "getSignalConditions") (param $self i32) (result i32)
    ;; Retrieve the struct field value from the correct offset
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $Signal::traits::hash (param $self i32) (param $state i32) (result i32)
    (local.get $state)
    ;; Hash the struct field values
    (call $Signal::get::conditions (local.get $self))
    (call $Hash::write_term))

  (func $Signal::traits::is_static (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Signal::traits::is_atomic (param $self i32) (result i32)
    (call $Signal::traits::is_static (local.get $self)))

  (func $Signal::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Signal::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; Compare the struct field values
    (call $Term::traits::equals (call $Signal::get::conditions (local.get $self)) (call $Signal::get::conditions (local.get $other))))

  (func $Signal::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $Signal::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Short-circuit signals encountered in the target position
    (local.get $self)
    (global.get $NULL))

  (func $Signal::traits::union (param $self i32) (param $other i32) (result i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $self))
      (then
        (local.get $other))
      (else
        (if (result i32)
          (i32.eq (global.get $NULL) (local.get $other))
          (then
            (local.get $self))
          (else
            (call $Signal::new
              (call $Tree::new
                (call $Signal::get::conditions (local.get $self))
                (call $Signal::get::conditions (local.get $other)))))))))

  (func $Signal::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $Tree::traits::collect (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (call $Signal::new)
    (local.get $dependencies))

  (func $Signal::partition_conditions_by_type (param $self i32) (param $condition_type i32) (result i32 i32)
    (local $conditions i32)
    (local $num_conditions i32)
    (local $partition i32)
    (local $iterator i32)
    (local $iterator_state i32)
    (local $item i32)
    (if (result i32 i32)
      (i32.eqz (local.tee $num_conditions (call $Tree::traits::length (local.tee $conditions (call $Signal::get::conditions (local.get $self))))))
      (then
        (call $List::empty)
        (call $List::empty))
      (else
        (local.tee $partition (call $List::allocate_partition_list (local.get $num_conditions)))
        (local.set $iterator (call $Tree::traits::iterate (local.get $conditions)))
        (local.set $iterator_state (global.get $NULL))
        (loop $LOOP
          (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (global.get $NULL))
          ;; Iterating a static Tree list should never produce dependencies, so it should be safe to drop them
          (drop)
          (local.set $iterator_state)
          (if
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              (call $List::insert_partition_list_item
                (local.get $partition)
                (i32.ne (local.get $condition_type) (call $Condition::get::type (local.get $item)))
                (local.get $item))
              (br $LOOP))))
        (call $List::init_partition_list_unordered)))))
