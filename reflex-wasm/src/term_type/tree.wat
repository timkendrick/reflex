;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Tree
    (@struct $Tree
      (@field $left (@ref $Term @optional))
      (@field $right (@ref $Term @optional))
      (@field $length i32)
      (@field $depth i32))

    (@derive $size (@get $Tree))
    (@derive $hash (@get $Tree))

    (@export $Tree (@get $Tree)))

  (export "isTree" (func $Term::Tree::is))

  (@const $Term::Tree::EMPTY i32
    (call $Term::TermType::Tree::new (global.get $NULL) (global.get $NULL) (i32.const 0) (i32.const 0)))

  (func $Tree::traits::equals (param $self i32) (param $other i32) (result i32)
    ;; This assumes that trees with the same length, depth and hash are almost certainly identical
    (i32.and
      (i32.eq
        (call $Tree::get::length (local.get $self))
        (call $Tree::get::length (local.get $other)))
      (i32.eq
        (call $Tree::get::depth (local.get $self))
        (call $Tree::get::depth (local.get $other)))))

  (func $Term::Tree::new (export "createTree") (param $left i32) (param $right i32) (result i32)
    (call $Term::TermType::Tree::new
      (local.get $left)
      (local.get $right)
      (i32.add
        (call $Term::Tree::get_branch_length (local.get $left))
        (call $Term::Tree::get_branch_length (local.get $right)))
      (call $Utils::i32::max_u
        (call $Term::Tree::get_branch_depth (local.get $left))
        (call $Term::Tree::get_branch_depth (local.get $right)))))

  (func $Term::Tree::empty (export "createEmptyTree") (result i32)
    (global.get $Term::Tree::EMPTY))

  (func $Term::Tree::of (export "createUnitTree") (param $value i32) (result i32)
    (call $Term::Tree::new (local.get $value) (global.get $NULL)))

  (func $Term::Tree::traits::is_atomic (param $self i32) (result i32)
    (local $branch i32)
    (i32.and
      (if (result i32)
        (i32.eq (global.get $NULL) (local.tee $branch (call $Term::Tree::get::left (local.get $self))))
        (then
          (global.get $TRUE))
        (else
          (call $Term::traits::is_atomic (local.get $branch))))
      (if (result i32)
        (i32.eq (global.get $NULL) (local.tee $branch (call $Term::Tree::get::right (local.get $self))))
        (then
          (global.get $TRUE))
        (else
          (call $Term::traits::is_atomic (local.get $branch))))))

  (func $Term::Tree::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $branch i32)
    ;; Write the opening parenthesis to the output
    (@store-bytes $offset "(")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the left branch to the output
    (local.set $offset
      (call $Term::traits::debug (call $Term::Tree::get::left (local.get $self)) (local.get $offset)))
    ;; Write the pair separator to the output
    (@store-bytes $offset " . ")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the right branch to the output
    (local.set $offset
      (call $Term::traits::debug (call $Term::Tree::get::right (local.get $self)) (local.get $offset)))
    ;; Write the closing parenthesis to the output and return the updated offset
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Tree::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Tree::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Tree::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $left i32)
    (local $right i32)
    (local $substituted_left i32)
    (local $substituted_right i32)
    (local.set $substituted_left
      (if (result i32)
        (i32.ne (local.tee $left (call $Term::Tree::get::left (local.get $self))) (global.get $NULL))
        (then
          (call $Term::traits::substitute
            (local.get $left)
            (local.get $variables)
            (local.get $scope_offset)))
        (else
          (global.get $NULL))))
    (local.set $substituted_right
      (if (result i32)
        (i32.ne (local.tee $right (call $Term::Tree::get::right (local.get $self))) (global.get $NULL))
        (then
          (call $Term::traits::substitute
            (local.get $right)
            (local.get $variables)
            (local.get $scope_offset)))
        (else
          (global.get $NULL))))
    (if (result i32)
      (i32.and
        (i32.eq (global.get $NULL) (local.get $substituted_left))
        (i32.eq (global.get $NULL) (local.get $substituted_right)))
      (then
        (global.get $NULL))
      (else
        (call $Term::Tree::new
          (select
            (call $Term::Tree::get::left (local.get $self))
            (local.get $substituted_left)
            (i32.eq (global.get $NULL) (local.get $substituted_left)))
          (select
            (call $Term::Tree::get::right (local.get $self))
            (local.get $substituted_right)
            (i32.eq (global.get $NULL) (local.get $substituted_right)))))))

  (func $Term::Tree::traits::union (param $self i32) (param $other i32) (result i32)
    (if (result i32)
      (i32.or
        (i32.or
          (i32.eq (global.get $NULL) (local.get $self))
          (i32.eq (call $Term::Tree::empty) (local.get $self)))
        (i32.or
          (i32.eq (global.get $NULL) (local.get $other))
          (i32.eq (call $Term::Tree::empty) (local.get $other))))
      (then
        (select
          (local.get $other)
          (local.get $self)
          (i32.or
            (i32.eq (global.get $NULL) (local.get $self))
            (i32.eq (call $Term::Tree::empty) (local.get $self)))))
      (else
        (call $Term::Tree::new (local.get $self) (local.get $other)))))

  (func $Term::Tree::traits::length (param $self i32) (result i32)
    (call $Term::Tree::get::length (local.get $self)))

  (func $Term::Tree::traits::iterate (param $self i32) (result i32)
    (local.get $self))

  (func $Term::Tree::traits::size_hint (param $self i32) (result i32)
    (call $Term::Tree::traits::length (local.get $self)))

  (func $Term::Tree::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Term::Tree::next_branch
      (if (result i32)
        (i32.eq (global.get $NULL) (local.get $iterator_state))
        (then
          ;; If this is the first iteration, allocate a new cell to hold the iteration state
          (call $Term::Tree::allocate_iterator_state (local.get $self)))
        (else
          ;; Otherwise use the state that was passed in from the previous iteration result
          (local.get $iterator_state))))
    (global.get $NULL))

  (func $Term::Tree::next_branch (param $iterator_state i32) (result i32 i32)
    (local $cursor i32)
    (local $current i32)
    (local $is_right i32)
    (if (result i32 i32)
      ;; If the stack has been fully emptied, dispose of the temporary iterator state and return the complete marker
      (i32.eq
        (i32.const -1)
        (local.tee $cursor
          (call $Term::Tree::get_iterator_state_cursor (local.get $iterator_state))))
      (then
        (call $Term::drop (local.get $iterator_state))
        (global.get $NULL)
        (global.get $NULL))
      (else
        ;; Get the current stack entry, and whether we are processing the left or right branch
        (call $Term::Tree::get_pair_iterator_stack_entry (local.get $iterator_state) (local.get $cursor))
        (local.set $is_right)
        (local.set $current)
        ;; Determine how to proceed depending on whether we are processing the left or right branch
        (if (result i32 i32)
          (local.get $is_right)
          (then
            (call $Term::Tree::next_right (call $Term::Tree::get::right (local.get $current)) (local.get $iterator_state)))
          (else
            (call $Term::Tree::next_left (call $Term::Tree::get::left (local.get $current)) (local.get $iterator_state)))))))

  (func $Term::Tree::next_left (param $left i32) (param $iterator_state i32) (result i32 i32)
    (if (result i32 i32)
      ;; If this is the null leaf marker, we need to shift from the left to the right branch
      (i32.eq (global.get $NULL) (local.get $left))
      (then
        ;; Continue with the right branch
        (call $Term::Tree::next_branch
          (call $Term::Tree::iterator_state_left_next (local.get $iterator_state))))
      (else
        (if (result i32 i32)
          ;; Determine whether the current item is itself a cell which needs to be traversed deeper
          (call $Term::Tree::is (local.get $left))
          (then
            ;; If so, push the cell to the stack and repeat the iteration with the updated stack
            (call $Term::Tree::next_branch
              (call $Term::Tree::iterator_state_push (local.get $iterator_state) (local.get $left))))
          (else
            ;; Otherwise emit the value
            (local.get $left)
            ;; Shift from the left to the right branch
            (call $Term::Tree::iterator_state_left_next (local.get $iterator_state)))))))

  (func $Term::Tree::next_right (param $right i32) (param $iterator_state i32) (result i32 i32)
    (local $next_cursor i32)
    (if (result i32 i32)
      ;; If this is the null leaf marker, we are at the end of a list and need to pop the stack
      (i32.eq (global.get $NULL) (local.get $right))
      (then
        (if (result i32 i32)
          ;; Pop the current entry from the stack; if this was the final item, dispose of the temporary iterator state and return the complete marker
          (i32.eq (i32.const -1) (local.tee $next_cursor (call $Term::Tree::iterator_state_pop (local.get $iterator_state))))
          (then
            (call $Term::drop (local.get $iterator_state))
            (global.get $NULL)
            (global.get $NULL))
          (else
            ;; Otherwise repeat the iteration with the updated stack
            (call $Term::Tree::next_branch (local.get $iterator_state)))))
      (else
        (if (result i32 i32)
          ;; Determine whether the current item is itself a cell which needs to be traversed deeper
          (call $Term::Tree::is (local.get $right))
          (then
            ;; Push the cell to the stack and repeat the iteration with the updated stack item
            (call $Term::Tree::next_branch
              (call $Term::Tree::iterator_state_push (local.get $iterator_state) (local.get $right))))
          (else
            ;; Pop the current entry from the stack
            (call $Term::Tree::iterator_state_pop (local.get $iterator_state))
            (drop)
            ;; Emit the value and state
            (local.get $right)
            (local.get $iterator_state))))))

  (func $Term::Tree::allocate_iterator_state (param $self i32) (result i32)
    (local $iterator_state i32)
    (local.tee $iterator_state (call $Term::Cell::allocate (i32.add (i32.const 1) (call $Term::Tree::get_depth (local.get $self)))))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (call $Term::Tree::create_iterator_state_cursor (i32.const 0) (i32.const 0)))
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 1) (local.get $self)))

  (func $Term::Tree::get_iterator_state_cursor (param $iterator_state i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.const 0)))

  (func $Term::Tree::set_iterator_state_cursor (param $iterator_state i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.const 0) (local.get $value)))

  (func $Term::Tree::get_iterator_state_stack_item (param $iterator_state i32) (param $index i32) (result i32)
    (call $Term::Cell::get_field (local.get $iterator_state) (i32.add (i32.const 1) (local.get $index))))

  (func $Term::Tree::set_iterator_state_stack_item (param $iterator_state i32) (param $index i32) (param $value i32)
    (call $Term::Cell::set_field (local.get $iterator_state) (i32.add (i32.const 1) (local.get $index)) (local.get $value)))

  (func $Term::Tree::create_iterator_state_cursor (param $depth i32) (param $is_right i32) (result i32)
    (i32.add (i32.mul (local.get $depth) (i32.const 2)) (local.get $is_right)))

  (func $Term::Tree::get_iterator_state_cursor_index (param $cursor i32) (result i32)
    (i32.div_u (local.get $cursor) (i32.const 2)))

  (func $Term::Tree::get_iterator_state_cursor_is_right (param $cursor i32) (result i32)
    (i32.rem_u (local.get $cursor) (i32.const 2)))

  (func $Term::Tree::get_pair_iterator_stack_entry (param $iterator_state i32) (param $cursor i32) (result i32 i32)
    (call $Term::Tree::get_iterator_state_stack_item
      (local.get $iterator_state)
      (call $Term::Tree::get_iterator_state_cursor_index (local.get $cursor)))
    (call $Term::Tree::get_iterator_state_cursor_is_right (local.get $cursor)))

  (func $Term::Tree::iterator_state_left_next (param $iterator_state i32) (result i32)
    (local $cursor i32)
    ;; Shift the cursor from the left to the right branch
    (call $Term::Tree::set_iterator_state_cursor
      (local.get $iterator_state)
      (i32.add (local.tee $cursor (call $Term::Tree::get_iterator_state_cursor (local.get $iterator_state))) (i32.const 1)))
    (local.get $iterator_state))

  (func $Term::Tree::iterator_state_push (param $iterator_state i32) (param $item i32) (result i32)
    (local $cursor i32)
    ;; Determine the stack offset where we should store the cell
    (call $Term::Tree::set_iterator_state_cursor
      (local.get $iterator_state)
      (local.tee $cursor
        ;; If we were processing the right branch, the cell is no longer needed so the current stack entry can be reused
        ;; If we were processing the left branch, create a new stack entry so that we can return later to process the right branch
        (i32.add
          (local.tee $cursor (call $Term::Tree::get_iterator_state_cursor (local.get $iterator_state)))
          (select
            (i32.const -1)
            (i32.const 2)
            (call $Term::Tree::get_iterator_state_cursor_is_right (local.get $cursor))))))
    ;; Store the cell at the given offset
    (call $Term::Tree::set_iterator_state_stack_item
      (local.get $iterator_state)
      (call $Term::Tree::get_iterator_state_cursor_index (local.get $cursor))
      (local.get $item))
    (local.get $iterator_state))

  (func $Term::Tree::iterator_state_pop (param $iterator_state i32) (result i32)
    (local $cursor i32)
    (call $Term::Tree::set_iterator_state_cursor
      (local.get $iterator_state)
      ;; The stack is only ever popped when processing the right branch; its parent's left branch must already have been processed
      ;; so it's safe to assume that we always jump from processing a child's right branch to the parent's right branch.
      ;; When popping the final entry from the stack this will result in the cursor being set to -1.
      (local.tee $cursor
        (i32.sub
          (local.tee $cursor (call $Term::Tree::get_iterator_state_cursor (local.get $iterator_state)))
          (i32.const 2))))
    (local.get $cursor))

  (func $Term::Tree::traits::values (param $self i32) (result i32)
    (call $Term::Tree::traits::iterate (local.get $self)))

  (func $Term::Tree::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (if (result i32 i32)
      ;; If the iterator is already a tree iterator, return the existing instance
      (call $Term::Tree::is (local.get $iterator))
      (then
        (local.get $iterator)
        (global.get $NULL))
      (else
        ;; Otherwise collect the iterator items into a tree
        (local.set $instance (global.get $NULL))
        (local.set $iterator_state (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (loop $LOOP
          ;; Consume the next item from the source iterator
          (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          (local.set $iterator_state)
          ;; If the iterator has been fully consumed, nothing more to do
          (if
            (i32.eq (local.tee $item) (global.get $NULL))
            (then)
            (else
              ;; Otherwise update the tree result and continue with the next item
              (local.set $instance (call $Term::Tree::new (local.get $item) (local.get $instance)))
              (br $LOOP))))
        ;; If the iterator produced no items, return the empty tree
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.get $instance))
          (then
            (call $Term::Tree::empty)
            (local.get $dependencies))
          (else
            ;; Otherwise return the tree
            (local.get $instance)
            (local.get $dependencies))))))

  (func $Term::Tree::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (local $item i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local $signal i32)
    ;; Collect the iterator items into a tree
    (local.set $instance (global.get $NULL))
    (local.set $iterator_state (global.get $NULL))
    (local.set $dependencies (global.get $NULL))
    (local.set $signal (global.get $NULL))
    (loop $LOOP
      ;; Consume the next item from the source iterator
      (call $Term::traits::next (local.get $iterator) (local.get $iterator_state) (local.get $state))
      (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
      (local.set $iterator_state)
      ;; If the iterator has been fully consumed, nothing more to do
      (if
        (i32.eq (local.tee $item) (global.get $NULL))
        (then)
        (else
          ;; Otherwise if a signal was encountered, update the combined signal result
          (if
            (i32.ne
              (global.get $NULL)
              (local.tee $signal
                (call $Term::Signal::traits::union
                  (local.get $signal)
                  (select
                    (local.get $item)
                    (global.get $NULL)
                    (call $Term::Signal::is (local.get $item))))))
            (then)
            (else
              ;; Otherwise update the tree result
              (local.set $instance (call $Term::Tree::new (local.get $item) (local.get $instance)))))
          ;; Continue with the next item
          (br $LOOP))))
    ;; If a signal was encountered during the iteration, return the signal
    (if (result i32 i32)
      (i32.ne (global.get $NULL) (local.get $signal))
      (then
        (local.get $signal)
        (local.get $dependencies))
      (else
        ;; Otherwise if the iterator produced no items, return the empty tree
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.get $instance))
          (then
            (call $Term::Tree::empty)
            (local.get $dependencies))
          (else
            ;; Otherwise return the tree
            (local.get $instance)
            (local.get $dependencies))))))

  (func $Term::Tree::get_left (export "getTreeLeft") (param $self i32) (result i32)
    (call $Term::Tree::get::left (local.get $self)))

  (func $Term::Tree::get_right (export "getTreeRight") (param $self i32) (result i32)
    (call $Term::Tree::get::right (local.get $self)))

  (func $Term::Tree::get_length (export "getTreeLength") (param $self i32) (result i32)
    (call $Term::Tree::get::length (local.get $self)))

  (func $Term::Tree::get_depth (export "getTreeDepth") (param $self i32) (result i32)
    (call $Term::Tree::get::depth (local.get $self)))

  (func $Term::Tree::get_branch_length (param $branch i32) (result i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $branch))
      (then
        (i32.const 0))
      (else
        (if (result i32)
          (call $Term::Tree::is (local.get $branch))
          (then
            (call $Term::Tree::get::length (local.get $branch)))
          (else
            (i32.const 1))))))

  (func $Term::Tree::get_branch_depth (param $branch i32) (result i32)
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $branch))
      (then
        (i32.const 0))
      (else
        (if (result i32)
          (call $Term::Tree::is (local.get $branch))
          (then
            (i32.add (i32.const 1) (call $Term::Tree::get_depth (local.get $branch))))
          (else
            (i32.const 1)))))))
