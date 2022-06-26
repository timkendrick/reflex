;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Constructor
    (@struct $Constructor
      (@field $keys (@ref $Term)))

    (@derive $size (@get $Constructor))
    (@derive $equals (@get $Constructor))
    (@derive $hash (@get $Constructor))

    (@export $Constructor (@get $Constructor)))

  (export "isConstructor" (func $Term::Constructor::is))
  (export "getConstructorKeys" (func $Term::Constructor::get::keys))

  (@const $Term::Constructor::EMPTY i32 (@depends-on $Term::List::EMPTY)
    (call $Term::TermType::Constructor::new (call $Term::List::empty)))

  (func $Term::Constructor::new (export "createConstructor") (param $keys i32) (result i32)
    (local $self i32)
    (if (result i32)
      (i32.eq (call $Term::List::traits::length (local.get $keys)) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::Constructor::EMPTY))
      (else
        (call $Term::TermType::Constructor::new
          (local.get $keys)))))

  (func $Term::Constructor::empty (result i32)
    (global.get $Term::Constructor::EMPTY))

  (func $Term::Constructor::traits::is_atomic (param $self i32) (result i32)
    (call $Term::List::traits::is_atomic (call $Term::Constructor::get::keys (local.get $self))))

  (func $Term::Constructor::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Constructor::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_keys i32)
    (local.set $substituted_keys
      (call $Term::traits::substitute
        (call $Term::Constructor::get::keys (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_keys))
      (then
        (global.get $NULL))
      (else
        (call $Term::Constructor::new
          (local.get $substituted_keys)))))

  (func $Term::Constructor::traits::arity (param $self i32) (result i32)
    (call $Term::List::get_length (call $Term::Constructor::get::keys (local.get $self))))

  (func $Term::Constructor::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    (local $keys i32)
    (if (result i32 i32)
      (i32.ne
        (call $Term::List::get_length (local.tee $keys (call $Term::Constructor::get::keys (local.get $self))))
        (call $Term::List::get_length (local.get $args)))
      (then
        (call $Term::Signal::of (call $Term::Condition::invalid_function_args (local.get $self) (local.get $args)))
        (global.get $NULL))
      (else
        (call $Term::Record::new (local.get $keys) (local.get $args))
        (global.get $NULL))))

  (func $Term::Constructor::parse_record (param $self i32) (param $properties i32) (result i32)
    (local $keys i32)
    (local $values i32)
    (local $num_keys i32)
    (local $index i32)
    (if (result i32)
      ;; If the input record contains identical field layout to the constructor, return it as-is
      (call $Term::traits::equals
        (local.tee $keys (call $Term::Constructor::get::keys (local.get $self)))
        (call $Term::Record::get_keys (local.get $properties)))
      (then
        (local.get $properties))
      (else
        ;; Otherwise determine whether the input record contains all the required fields
        (local.set $num_keys (call $Term::List::get_length (local.get $keys)))
        ;; Iterate through each of the constructor fields to determine whether they exist on the input record
        (loop $LOOP
          (if
            ;; Determine whether the input record contains the current field
            (call $Term::Record::traits::has
              (local.get $properties)
              (call $Term::List::get_item (local.get $keys) (local.get $index)))
            (then
              ;; If the input record contains the field, continue with the next field
              (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_keys))))
            ;; Otherwise return the null sentinel value
            (else
              (return (global.get $NULL)))))
        ;; Reset the iteration index
        (local.set $index (i32.const 0))
        ;; Push the constructor keys onto the stack (used to construct the record later)
        (local.get $keys)
        ;; Allocate a new list to hold the correctly-ordered values
        (local.tee $values (call $Term::List::allocate (local.get $num_keys)))
        ;; Iterate through each of the constructor fields and copy the corresponding value into the newly-allocated list
        (loop $LOOP
          ;; Store the corresponsing value for the current field in the output list
          (call $Term::List::set_item
            (local.get $values)
            (local.get $index)
            (call $Term::Record::traits::get
              (local.get $properties)
              (call $Term::List::get_item (local.get $keys) (local.get $index))))
          ;; Continue with the next field
          (br_if $LOOP (i32.lt_u (local.tee $index (i32.add (i32.const 1) (local.get $index))) (local.get $num_keys))))
        ;; Instantiate the list of field values
        (call $Term::List::init (local.get $num_keys))
        ;; Create a record from the keys and values lists currently at the top of the stack
        (call $Term::Record::new)))))
