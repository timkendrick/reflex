;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  ;; TODO: Compile singleton instances directly into linear memory data
  (global $ChainIterator::EMPTY (mut i32) (i32.const -1))

  (func $ChainIterator::startup
    ;; Pre-allocate the singleton instance
    (call $Term::new (global.get $TermType::ChainIterator) (i32.const 0))
    (call $ChainIterator::init)
    ;; Update the global variable with a pointer to the singleton instance
    (global.set $ChainIterator::EMPTY))

  (func $ChainIterator::allocate (export "allocateChainIterator") (param $num_sources i32) (result i32)
    (if (result i32)
      (i32.eqz (local.get $num_sources))
      (then
        (global.get $ChainIterator::EMPTY))
      (else
        (call $Term::new (global.get $TermType::ChainIterator) (local.get $num_sources)))))

  (func $ChainIterator::init (export "initChainIterator") (param $self i32) (result i32)
    (local.get $self)
    (call $Term::init))

  (func $ChainIterator::create_pair (export "createChainPairIterator") (param $left i32) (param $right i32) (result i32)
    ;; TODO: Flatten nested chain iterators into single iterator
    (local $self i32)
    (local.tee $self (call $ChainIterator::allocate (i32.const 2)))
    (call $ChainIterator::set_source (local.get $self) (i32.const 0) (local.get $left))
    (call $ChainIterator::set_source (local.get $self) (i32.const 1) (local.get $right))
    (call $ChainIterator::init))

  (func $ChainIterator::is (export "isChainIterator") (param $self i32) (result i32)
    (i32.eq (global.get $TermType::ChainIterator) (call $Term::get_type (local.get $self))))

  (func $ChainIterator::get::sources (export "getChainIteratorSources") (param $self i32) (param $source_index i32) (result i32)
    (call $Term::get_field (local.get $self) (i32.const 0)))

  (func $ChainIterator::traits::is_static (param $self i32) (result i32)
    (global.get $TRUE))

  (func $ChainIterator::traits::is_atomic (param $self i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (if (result i32)
      (i32.eqz (local.tee $num_sources (call $ChainIterator::get_num_sources (local.get $self))))
      (then
        (global.get $TRUE))
      (else
        (loop $LOOP
          (if
            (i32.eqz (call $Term::traits::is_atomic (call $ChainIterator::get_source (local.get $self) (local.get $index))))
            (then
              (return (global.get $FALSE)))
            (else
              (br_if $LOOP
                (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))))
        (global.get $TRUE))))

  (func $ChainIterator::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $ChainIterator::traits::hash (param $self i32) (param $state i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (if (result i32)
      (i32.eqz (local.tee $num_sources (call $ChainIterator::get_num_sources (local.get $self))))
      (then
        (local.get $state))
      (else
        (loop $LOOP
          (local.set $state (call $Hash::write_term (local.get $state) (call $ChainIterator::get_source (local.get $self) (local.get $index))))
          (br_if $LOOP
            (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))
        (local.get $state))))

  (func $ChainIterator::traits::equals (param $self i32) (param $other i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (if (result i32)
      (i32.ne
        (local.tee $num_sources (call $ChainIterator::get_num_sources (local.get $self)))
        (call $ChainIterator::get_num_sources (local.get $other)))
      (then
        (global.get $FALSE))
      (else
        (if (result i32)
          (i32.eqz (local.get $num_sources))
          (then
            (global.get $TRUE))
          (else
            (loop $LOOP
              (if
                (call $Term::traits::equals
                  (call $ChainIterator::get_source (local.get $self) (local.get $index))
                  (call $ChainIterator::get_source (local.get $other) (local.get $index)))
                (then
                  (br_if $LOOP
                    (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))
                (else
                  (return (global.get $FALSE)))))
            (global.get $TRUE))))))

  (func $ChainIterator::traits::write_json (param $self i32) (param $offset i32) (result i32)
    (call $Term::traits::write_json (call $Record::empty) (local.get $offset)))

  (func $ChainIterator::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $ChainIterator::traits::size_hint (param $self i32) (result i32)
    (local $index i32)
    (local $num_sources i32)
    (local $source_size i32)
    (local $combined_size i32)
    (if (result i32)
      (i32.eqz (local.tee $num_sources (call $ChainIterator::get_num_sources (local.get $self))))
      (then
        (i32.const 0))
      (else
        (loop $LOOP
          (if
            (i32.eq
              (global.get $NULL)
              (local.tee $source_size
                (call $Term::traits::size_hint (call $ChainIterator::get_source (local.get $self) (local.get $index)))))
            (then
              (return (global.get $NULL)))
            (else
              (local.set $combined_size (i32.add (local.get $combined_size) (local.get $source_size)))
              (br_if $LOOP
                (i32.ne (local.get $num_sources) (local.tee $index (i32.add (local.get $index) (i32.const 1))))))))
        (local.get $combined_size))))

  (func $ChainIterator::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
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
          (call $ChainIterator::allocate_iterator_state))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    (local.set $source_index (call $ChainIterator::get_iterator_state_source_index (local.get $iterator_state)))
    (local.set $inner_state (call $ChainIterator::get_iterator_state_inner_state (local.get $iterator_state)))
    (local.set $source (call $ChainIterator::get_source (local.get $self) (local.get $source_index)))
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
          (i32.lt_u (local.get $source_index) (call $ChainIterator::get_num_sources (local.get $self)))
          (then
            (call $ChainIterator::initialize_iterator_state (local.get $iterator_state) (i32.add (local.get $source_index) (i32.const 1)))
            (call $ChainIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          (else
            ;; Otherwise dispose of the temporary iteration state cell and return the complete marker
            (call $Term::drop (local.get $iterator_state))
            (global.get $NULL)
            (global.get $NULL)
            (local.get $dependencies))))
      (else
        ;; Update the iteration state
        (call $ChainIterator::update_iterator_state_inner_state (local.get $iterator_state) (local.get $inner_state))
        ;; Emit the value and the updated state
        (local.get $value)
        (local.get $iterator_state)
        (local.get $dependencies))))

  (func $ChainIterator::get_num_sources (export "getChainIteratorNumSources") (param $self i32)  (result i32)
    (call $Term::get_num_fields (local.get $self)))

  (func $ChainIterator::get_source (export "getChainIteratorSource") (param $self i32) (param $source_index i32) (result i32)
    (call $Term::get_field (local.get $self) (local.get $source_index)))

  (func $ChainIterator::set_source (export "setChainIteratorSource") (param $self i32) (param $source_index i32) (param $value i32)
    (call $Term::set_field (local.get $self) (local.get $source_index) (local.get $value)))

  (func $ChainIterator::allocate_iterator_state (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Cell::new (i32.const 2)))
    (call $ChainIterator::initialize_iterator_state (local.get $iterator_state) (i32.const 0)))

  (func $ChainIterator::initialize_iterator_state (param $iterator_state i32) (param $source_index i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $source_index))
    (call $Cell::set_field (local.get $iterator_state) (i32.const 1) (global.get $NULL)))

  (func $ChainIterator::get_iterator_state_source_index (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $ChainIterator::get_iterator_state_inner_state (param $iterator_state i32) (result i32)
    (call $Cell::get_field (local.get $iterator_state) (i32.const 1)))

  (func $ChainIterator::update_iterator_state_inner_state (param $iterator_state i32) (param $value i32)
    (call $Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $value))))
