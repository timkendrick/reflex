;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $ChainIterator
    (@struct $ChainIterator
      (@field $sources (@repeated (@ref $Term))))

    (@derive $size (@get $ChainIterator))
    (@derive $equals (@get $ChainIterator))
    (@derive $hash (@get $ChainIterator))

    (@export $ChainIterator (@get $ChainIterator)))

  (export "isChainIterator" (func $Term::ChainIterator::is))
  (export "getChainIteratorSources" (func $Term::ChainIterator::pointer::sources))
  (export "setChainIteratorSource" (func $Term::ChainIterator::set::sources::value))

  ;; TODO: Compile singleton instances directly into linear memory data
  (global $Term::ChainIterator::EMPTY (mut i32) (i32.const -1))

  (func $Term::ChainIterator::startup
    ;; Pre-allocate the singleton instances
    (global.set $Term::ChainIterator::EMPTY (call $Term::TermType::ChainIterator::new)))

  (func $Term::ChainIterator::empty::sizeof (result i32)
    ;; Determine the size of the term wrapper by inspecting the sources pointer for an imaginary chain term located at
    ;; memory address 0. The pointer offset tells us how many bytes are taken up by the preceding chain wrapper.
    (call $Term::ChainIterator::get::sources::pointer (i32.const 0) (i32.const 0)))

  (func $Term::ChainIterator::allocate (export "allocateChainIterator") (param $num_sources i32) (result i32)
    ;; Allocates a new ChainIterator term with the given capacity, allowing items to be copied directly into the
    ;; allocated slots.
    ;; The term must be instantiated before it can be used.
    (local $self i32)
    (if (result i32)
      (i32.eq (local.get $num_sources) (i32.const 0))
      (then
        ;; Return the pre-allocated singleton instance
        (global.get $Term::ChainIterator::EMPTY))
      (else
        ;; The standard constructor wrappers take care of allocating space for a standard term,
        ;; however they do not allocate space for extra elements as needed by the chain iterator term.
        ;; This means we have to manually allocate a larger amount of space than usual,
        ;; then fill in the chain iterator term contents into the newly-allocated space.
        ;; First allocate a new term wrapper with the required capacity
        (local.tee $self
          (call $Allocator::allocate
            (i32.add
              (call $Term::ChainIterator::empty::sizeof)
              (i32.mul (i32.const 4) (local.get $num_sources)))))
        ;; Then manually write the chain iterator struct contents into the term wrapper
        (call $TermType::ChainIterator::construct (call $Term::pointer::value (local.get $self)))
        (call $Term::ChainIterator::set::sources::capacity (local.get $self) (local.get $num_sources)))))

  (func $Term::ChainIterator::init (export "initChainIterator") (param $self i32) (param $num_sources i32) (result i32)
    (call $Term::ChainIterator::set::sources::length (local.get $self) (local.get $num_sources))
    (call $Term::init (local.get $self)))

  (func $Term::ChainIterator::create_pair (export "createChainPairIterator") (param $left i32) (param $right i32) (result i32)
    ;; TODO: Flatten nested chain iterators into single iterator
    (local $self i32)
    (local.tee $self (call $Term::ChainIterator::allocate (i32.const 2)))
    (call $Term::ChainIterator::set::sources::value (local.get $self) (i32.const 0) (local.get $left))
    (call $Term::ChainIterator::set::sources::value (local.get $self) (i32.const 1) (local.get $right))
    (call $Term::ChainIterator::init (i32.const 2)))

  (func $Term::ChainIterator::traits::is_atomic (param $self i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (if (result i32)
      (i32.eqz (local.tee $num_sources (call $Term::ChainIterator::get::sources::length (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        (loop $LOOP
          (if
            (i32.eqz (call $Term::traits::is_atomic (call $Term::ChainIterator::get::sources::value (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              (br_if $LOOP
                (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))))
        (global.get $TRUE))))

  (func $Term::ChainIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::ChainIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Term::Record::empty) (local.get $offset)))

  (func $Term::ChainIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::ChainIterator::traits::size_hint (param $self i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (local $source_size i32)
    (local $combined_size i32)
    (if (result i32)
      (i32.eqz (local.tee $num_sources (call $Term::ChainIterator::get::sources::length (local.get $self))))
      (then
        (i32.const 0))
      (else
        (loop $LOOP
          (if
            (i32.eq
              (global.get $NULL)
              (local.tee $source_size
                (call $Term::traits::size_hint (call $Term::ChainIterator::get::sources::value (local.get $self) (local.get $index)))))
            (then
              (return (global.get $NULL)))
            (else
              (local.set $combined_size (i32.add (local.get $combined_size) (local.get $source_size)))
              (br_if $LOOP
                (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))))
        (local.get $combined_size))))

  (func $Term::ChainIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (local $source_index i32)
    (local $inner_state i32)
    (local $source i32)
    (local $value i32)
    (local $dependencies i32)
    ;; Initialize the iterator state
    (local.set $iterator_state
      (if (result i32)
        (i32.eq (local.get $iterator_state) (global.get $NULL))
        (then
          ;; If this is the first iteration, allocate a new cell to hold the iteration state
          (call $Term::ChainIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    (local.set $source_index (call $Term::ChainIterator::get_iterator_state_source_index (local.get $iterator_state)))
    (local.set $inner_state (call $Term::ChainIterator::get_iterator_state_inner_state (local.get $iterator_state)))
    (local.set $source (call $Term::ChainIterator::get::sources::value (local.get $self) (local.get $source_index)))
    ;; Get the next iterator item and updated state
    (call $Term::traits::next (local.get $source) (local.get $inner_state) (local.get $state))
    (local.set $dependencies)
    (local.set $inner_state)
    (if (result i32 i32 i32)
      ;; Check whether the source iterator has been fully consumed
      (i32.eq (local.tee $value) (global.get $NULL))
      (then
        (if (result i32 i32 i32)
          ;; If this was not the final source iterator, switch to the next one and try again
          (i32.lt_u
            (local.tee $source_index (i32.add (local.get $source_index) (i32.const 1)))
            (call $Term::ChainIterator::get::sources::length (local.get $self)))
          (then
            (call $Term::ChainIterator::initialize_iterator_state (local.get $iterator_state) (local.get $source_index))
            (call $Term::ChainIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          (else
            ;; Otherwise dispose of the temporary iteration state cell and return the complete marker
            (call $Term::drop (local.get $iterator_state))
            (global.get $NULL)
            (global.get $NULL)
            (local.get $dependencies))))
      (else
        ;; Update the iteration state
        (call $Term::ChainIterator::update_iterator_state_inner_state (local.get $iterator_state) (local.get $inner_state))
        ;; Emit the value and the updated state
        (local.get $value)
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $Term::ChainIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.const 2)))
    (call $Term::ChainIterator::initialize_iterator_state (local.get $iterator_state) (i32.const 0)))

  (func $Term::ChainIterator::initialize_iterator_state (param $iterator_state i32) (param $source_index i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $source_index))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (global.get $NULL)))

  (func $Term::ChainIterator::get_iterator_state_source_index (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $Term::ChainIterator::get_iterator_state_inner_state (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $Term::ChainIterator::update_iterator_state_inner_state (param $iterator_state i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $value))))
