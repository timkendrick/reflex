;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Signal
    (@struct $Signal
      (@field $conditions (@ref $Term)))

    (@derive $size (@get $Signal))
    (@derive $equals (@get $Signal))
    (@derive $hash (@get $Signal))

    (@export $Signal (@get $Signal)))

  (export "isSignal" (func $Term::Signal::is))
  (export "getSignalConditions" (func $Term::Signal::get::conditions))

  (@const $Term::Signal::PENDING i32 (@depends-on $Term::Condition::PENDING)
    (call $Term::TermType::Signal::new (call $Term::Tree::of (call $Term::Condition::pending))))
  (@const $Term::Signal::INVALID_POINTER i32 (@depends-on $Term::Condition::INVALID_POINTER)
    (call $Term::TermType::Signal::new (call $Term::Tree::of (call $Term::Condition::invalid_pointer))))

  (func $Term::Signal::of (export "createSignal") (param $condition i32) (result i32)
    (call $Term::TermType::Signal::new (call $Term::Tree::of (local.get $condition))))

  (func $Term::Signal::pending (export "createPendingSignal") (result i32)
    (global.get $Term::Signal::PENDING))

  (func $Term::Signal::invalid_pointer (result i32)
    (global.get $Term::Signal::INVALID_POINTER))

  (func $Term::Signal::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $Term::Tree::traits::collect (local.get $iterator) (local.get $state))
    (local.set $dependencies)
    (call $Term::TermType::Signal::new)
    (local.get $dependencies))

  (func $Term::Signal::traits::union (param $self i32) (param $other i32) (result i32)
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
            (call $Term::TermType::Signal::new
              (call $Term::Tree::new
                (call $Term::Signal::get::conditions (local.get $self))
                (call $Term::Signal::get::conditions (local.get $other)))))))))

  (func $Term::Signal::traits::is_atomic (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Signal::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Signal::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $conditions i32)
    (local $condition i32)
    (local $index i32)
    ;; Write the opening brace to the output
    (@store-bytes $offset "{")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the signal conditions to the output
    (local.set $offset
      (call $Term::Signal::display_tree_items
        (call $Term::Signal::get::conditions (local.get $self))
        (local.get $offset)
        (global.get $FALSE)))
    ;; Write the closing brace to the output and return the updated offset
    (@store-bytes $offset "}")
    (i32.add (local.get $offset)))

  (func $Term::Signal::display_tree_items (param $self i32) (param $offset i32) (param $separator i32) (result i32)
    (local $updated_offset i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $self))
      (then
        (local.get $offset))
      (else
        (if (result i32)
          (call $Term::Tree::is (local.get $self))
          (then
            (if
              (i32.ne
                (local.tee $updated_offset
                  (call $Term::Signal::display_tree_items
                    (call $Term::Tree::get_left (local.get $self))
                    (local.get $offset)
                    (local.get $separator)))
                (local.get $offset))
              (then
                (local.set $offset (local.get $updated_offset))
                (local.set $separator (global.get $TRUE))))
            (call $Term::Signal::display_tree_items
              (call $Term::Tree::get_right (local.get $self))
              (local.get $offset)
              (local.get $separator)))
          (else
            (if
              (local.get $separator)
              (then
                (@store-bytes $offset ",")
                (local.set $offset (i32.add (local.get $offset)))))
            (call $Term::traits::display (local.get $self) (local.get $offset)))))))

  (func $Term::Signal::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (global.get $NULL))

  (func $Term::Signal::traits::arity (param $self i32) (result i32)
    (i32.const 0))

  (func $Term::Signal::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Short-circuit signals encountered in the target position
    (local.get $self)
    (global.get $NULL))

  (func $Term::Signal::partition_conditions_by_type (param $self i32) (param $condition_type i32) (result i32 i32)
    (local $conditions i32)
    (local $num_conditions i32)
    (local $partition i32)
    (local $iterator_state i32)
    (local $item i32)
    (if (result i32 i32)
      (i32.eqz
        (local.tee $num_conditions
          (call $Term::Tree::traits::length
            (local.tee $conditions (call $Term::Signal::get::conditions (local.get $self))))))
      (then
        (call $Term::List::empty)
        (call $Term::List::empty))
      (else
        (local.tee $partition (call $Term::List::allocate_partition_list (local.get $num_conditions)))
        (local.set $iterator_state (global.get $NULL))
        (loop $LOOP
          (call $Term::traits::next (local.get $conditions) (local.get $iterator_state) (global.get $NULL))
          ;; Iterating a static Tree list should never produce dependencies, so it should be safe to drop them
          (drop)
          (local.set $iterator_state)
          (if
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              (call $Term::List::insert_partition_list_item
                (local.get $partition)
                (i32.ne (local.get $condition_type) (call $Term::Condition::get::type (local.get $item)))
                (local.get $item))
              (br $LOOP))))
        (call $Term::List::init_partition_list_unordered)))))
