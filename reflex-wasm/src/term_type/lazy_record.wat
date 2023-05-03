;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $LazyRecord
    (@struct $LazyRecord
      (@field $fields (@ref $Term))
      (@field $eagerness (@repeated i32)))

    (@derive $size (@get $LazyRecord))
    (@derive $equals (@get $LazyRecord))
    (@derive $hash (@get $LazyRecord))

    (@export $LazyRecord (@get $LazyRecord)))

  (export "isLazyRecord" (func $Term::LazyRecord::is))

  (func $Term::LazyRecord::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the list items pointer for an imaginary list term located at
    ;; memory address 0. The pointer offset tells us how many bytes are taken up by the preceding list wrapper.
    (call $Term::LazyRecord::get::eagerness::value (i32.const 0) (i32.const 0)))

  (func $Term::LazyRecord::new (param $fields i32) (param $source i32) (result i32)
    (local $self i32)
    (local $eagerness_length i32)
    (local.set $eagerness_length (call $Term::LazyRecord::get::eagerness::length (local.get $source)))
    ;; The standard constructor wrappers take care of allocating space for a standard term,
    ;; however they do not allocate space for extra elements as needed by the eagerness array.
    ;; This means we have to manually allocate a larger amount of space than usual,
    ;; then fill in the list term contents into the newly-allocated space.
    ;; First allocate a new term wrapper with the required capacity
    (local.tee $self
      (call $Allocator::allocate
        (i32.add
          (call $Term::LazyRecord::empty::sizeof)
          (i32.mul (i32.const 4) (local.get $eagerness_length)))))
    ;; Then manually write the lazy record struct contents into the term wrapper
    (call $TermType::LazyRecord::construct (call $Term::pointer::value (local.get $self)) (local.get $fields))
    ;; Copy the provided eagerness array into the newly-allocated term
    (call $Term::LazyRecord::set::eagerness::capacity (local.get $self) (local.get $eagerness_length))
    (call $Term::LazyRecord::set::eagerness::length (local.get $self) (local.get $eagerness_length))
    (memory.copy
      (call $Term::LazyRecord::get::eagerness::pointer (local.get $self) (i32.const 0))
      (call $Term::LazyRecord::get::eagerness::pointer (local.get $source) (i32.const 0))
      (i32.mul (i32.const 4) (local.get $eagerness_length))))

  (func $Term::LazyRecord::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::LazyRecord::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::LazyRecord::traits::display (param $self i32) (param $offset i32) (result i32)
    (call $Term::Record::traits::display
      (call $Term::LazyRecord::get::fields (local.get $self))
      (local.get $offset)))

  (func $Term::LazyRecord::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::LazyRecord::traits::display (local.get $self) (local.get $offset)))

  (func $Term::LazyRecord::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_fields i32)
    (local.set $substituted_fields
      (call $Term::traits::substitute
        (call $Term::LazyRecord::get::fields (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_fields))
      (then
        (global.get $NULL))
      (else
        (call $Term::LazyRecord::new (local.get $substituted_fields) (local.get $self)))))

  (func $Term::LazyRecord::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
    (call $Term::Record::traits::to_json
      (call $Term::LazyRecord::get::fields (local.get $self))
      (local.get $offset)))

  (func $Term::LazyRecord::traits::length (param $self i32) (result i32)
    (call $Term::Record::traits::length (call $Term::LazyRecord::get::fields (local.get $self))))

  (func $Term::LazyRecord::traits::iterate (param $self i32) (result i32)
    (call $Term::Record::traits::iterate (call $Term::LazyRecord::get::fields (local.get $self))))

  (func $Term::LazyRecord::traits::size_hint (param $self i32) (result i32)
    (call $Term::Record::traits::size_hint (call $Term::LazyRecord::get::fields (local.get $self))))

  (func $Term::LazyRecord::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Term::Record::traits::next
      (call $Term::LazyRecord::get::fields (local.get $self))
      (local.get $iterator_state)
      (local.get $state)))

  (func $Term::LazyRecord::traits::get (param $self i32) (param $key i32) (result i32)
    (call $Term::Record::traits::get
      (call $Term::LazyRecord::get::fields (local.get $self))
      (local.get $key)))

  (func $Term::LazyRecord::traits::has (param $self i32) (param $key i32) (result i32)
    (call $Term::Record::traits::has
      (call $Term::LazyRecord::get::fields (local.get $self))
      (local.get $key)))

  (func $Term::LazyRecord::traits::keys (param $self i32) (result i32)
    (call $Term::Record::traits::keys (call $Term::LazyRecord::get::fields (local.get $self))))

  (func $Term::LazyRecord::traits::values (param $self i32) (result i32)
    (call $Term::Record::traits::values (call $Term::LazyRecord::get::fields (local.get $self))))

  (func $Term::LazyRecord::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    ;; Push the keys onto the stack
    (call $Term::Record::get::keys (call $Term::LazyRecord::get::fields (local.get $self)))
    ;; Collect the values into a new list term
    (call $Term::LazyRecord::collect_field_values
      (local.get $self)
      (call $Term::Record::traits::length (call $Term::LazyRecord::get::fields (local.get $self)))
      (call $Term::Record::traits::values (call $Term::LazyRecord::get::fields (local.get $self)))
      (local.get $state))
    ;; Pop the dependencies from the stack, leaving the keys and values on the stack
    (local.set $dependencies)
    ;; Construct a new record term from the keys and values
    (call $Term::Record::new)
    (local.get $dependencies))

  (func $Term::LazyRecord::collect_field_values (param $self i32) (param $length i32) (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $index i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    (local $item_eagerness i32)
    (if (result i32 i32)
      ;; If the iterator is empty, return the empty list
      (i32.eqz (local.get $length))
      (then
        (call $Term::List::empty)
        (global.get $NULL))
      (else
        ;; Otherwise allocate a new list to hold the results and fill it by consuming each iterator item in turn
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $signal (global.get $NULL))
        (local.set $instance (call $Term::List::allocate (local.get $length)))
        (loop $LOOP
          ;; Consume the next iterator item
          (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
          ;; Update the accumulated dependencies and iterator state
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          (if
            ;; If the iterator has been fully consumed, nothing more to do
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              ;; Determine the eagerness of the current list item (0 = Lazy, 1 = Eager, 2 = Strict)
              ;; FIXME: Extract Eagerness into separate struct type
              (local.set $item_eagerness (call $Term::LazyRecord::get::eagerness::value (local.get $self) (local.get $index)))
              ;; If this is an eager item, resolve the list item
              (if
                (i32.ne (local.get $item_eagerness) (i32.const 0))
                (then
                  (call $Term::traits::evaluate (local.get $item) (local.get $state))
                  (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
                  (local.set $item)))
              (if
                ;; If the current item is a strict item that resolves to a signal, or a signal has already been encountered,
                ;; update the combined signal and continue with the next item
                (i32.ne
                  (global.get $NULL)
                  (local.tee $signal
                    (call $Term::Signal::traits::union
                      (local.get $signal)
                      (select
                        (local.get $item)
                        (global.get $NULL)
                        (i32.and
                          (i32.eq (local.get $item_eagerness) (i32.const 2))
                          (call $Term::Signal::is (local.get $item)))))))
                (then
                  ;; Continue with the next item
                  (local.set $index (i32.add (local.get $index) (i32.const 1)))
                  (br $LOOP))
                (else
                  ;; Otherwise store the item in the results list
                  (call $Term::List::set::items::value (local.get $instance) (local.get $index) (local.get $item))
                  ;; Continue with the next item
                  (local.set $index (i32.add (local.get $index) (i32.const 1)))
                  (br $LOOP))))))
        (if (result i32 i32)
          ;; If a signal was encountered during the iteration, return the combined signal
          (i32.ne (global.get $NULL) (local.get $signal))
          (then
            (local.get $signal)
            (local.get $dependencies))
          (else
            ;; Otherwise initialize the results list
            (call $Term::List::init (local.get $instance) (local.get $index))
            (local.get $dependencies)))))))
