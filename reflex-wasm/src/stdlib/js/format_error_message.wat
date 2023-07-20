;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Stdlib_FormatErrorMessage::MAX_MESSAGES i32 (i32.const 10))
  (@const $Stdlib_FormatErrorMessage::NEWLINE i32 (call $Term::String::from_char (@char "\n")))
  (@const-string $Stdlib_FormatErrorMessage::MESSAGE "message")
  (@const-string $Stdlib_FormatErrorMessage::NAME "name")

  (@builtin $Stdlib_FormatErrorMessage "FormatErrorMessage"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_FormatErrorMessage::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_FormatErrorMessage::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $name i32)
        (local $message i32)
        (local $name_length i32)
        (local $message_length i32)
        (local $offset i32)
        (local $result i32)
        (local $dependencies i32)
        (if (result i32 i32)
          ;; If the error object does not have a "message" field, format the error object as a string
          (i32.eq
            (local.tee $message (call $Stdlib_FormatErrorMessage::get_error_message (local.get $self)))
            (global.get $NULL))
          (then
            (call $Term::String::from(local.get $self))
            (global.get $NULL))
          (else
            ;; Otherwise evaluate the "message" field and convert the result to a string
            (call $Term::traits::evaluate (local.get $message) (local.get $state))
            (local.set $dependencies)
            (local.set $message (call $Term::String::from))
            ;; Determine whether the error object has a "name" field
            (if (result i32 i32)
              (i32.eq
                (local.tee $name (call $Stdlib_FormatErrorMessage::get_error_name (local.get $self)))
                (global.get $NULL))
              (then
                ;; If the error object does not have a "name" field, return the message as-is
                (local.get $message)
                (local.get $dependencies))
              (else
                ;; Otherwise if the error object has a "name" field, use it to prefix the "message field"
                (local.tee $result
                  (call $Term::String::allocate
                    (i32.add
                      (i32.add
                        (local.tee $name_length (call $Term::String::get_length (local.get $name)))
                        (i32.const 2))
                      (local.tee $message_length (call $Term::String::get_length (local.get $message))))))
                ;; Write the name to the output
                (local.set $offset (call $Term::String::get_char_offset (local.get $result) (i32.const 0)))
                (memory.copy
                  (local.get $offset)
                  (call $Term::String::get_char_offset (local.get $name) (i32.const 0))
                  (local.get $name_length))
                (local.set $offset (i32.add (local.get $offset) (local.get $name_length)))
                ;; Write a colon separator to the output
                (i32.store8 (local.get $offset) (@char ":"))
                (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
                (i32.store8 (local.get $offset) (@char " "))
                (local.set $offset (i32.add (local.get $offset) (i32.const 1)))
                ;; Write the message to the output
                (memory.copy
                  (local.get $offset)
                  (call $Term::String::get_char_offset (local.get $message) (i32.const 0))
                  (local.get $message_length))
                ;; Initialize the string term
                (call $Term::String::init)
                (local.get $dependencies)))))))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_FormatErrorMessage::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $messages i32)
        (local $num_errors i32)
        (local $dependencies i32)
        (call $Term::List::traits::collect_strict
          (call $Term::TakeIterator::new
            (call $Term::MapIterator::new
              (local.get $self)
              (call $Term::Builtin::new (global.get $Stdlib_FormatErrorMessage)))
            (global.get $Stdlib_FormatErrorMessage::MAX_MESSAGES))
          (local.get $state))
        (local.set $dependencies)
        (if (result i32 i32)
          ;; If there were fewer than the maximum number of displayed errors, display them as a combined output string
          (i32.lt_u
            (call $Term::List::get_length (local.tee $messages))
            (global.get $Stdlib_FormatErrorMessage::MAX_MESSAGES))
          (then
            (call $Stdlib_FormatErrorMessage::combine_error_messages (local.get $messages) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          (else
            ;; Otherwise determine the total number of errors
            (call $Stdlib_FormatErrorMessage::get_iterator_length (local.get $self) (local.get $state))
            (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
            (if (result i32 i32)
              ;; If there were exactly the maximum number of displayed errors, display them as a combined output string
              (i32.eq (local.tee $num_errors) (global.get $Stdlib_FormatErrorMessage::MAX_MESSAGES))
              (then
                (call $Stdlib_FormatErrorMessage::combine_error_messages (local.get $messages) (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies)))
              (else
                ;; Otherwise replace the final displayed message with an "...X more errors" label
                ;; Calculate the total number of undisplayed errors
                (i32.sub
                  (local.get $num_errors)
                  (i32.sub (global.get $Stdlib_FormatErrorMessage::MAX_MESSAGES) (i32.const 1)))
                ;; Format the number of undisplayed errors as a label string
                (local.set $num_errors
                  (call $Term::OnceIterator::new
                    (call $Stdlib_FormatErrorMessage::get_remaining_items_label)))
                ;; Return the combined error message
                (call $Stdlib_FormatErrorMessage::combine_error_messages
                  ;; Replace the final error message with the label string
                  (call $Term::FlattenIterator::new
                    (call $Term::List::create_pair
                      (call $Term::TakeIterator::new
                        (local.get $messages)
                        (i32.sub (global.get $Stdlib_FormatErrorMessage::MAX_MESSAGES) (i32.const 1)))
                      (local.get $num_errors)))
                  (local.get $state))
                (call $Dependencies::traits::union (local.get $dependencies))))))))

    (@default
      (func $Stdlib_FormatErrorMessage::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::String::from (local.get $self))
        (global.get $NULL))))

  (func $Stdlib_FormatErrorMessage::get_error_name (param $error i32) (result i32)
    (call $Term::Record::traits::get (local.get $error) (global.get $Stdlib_FormatErrorMessage::NAME)))

  (func $Stdlib_FormatErrorMessage::get_error_message (param $error i32) (result i32)
    (call $Term::Record::traits::get (local.get $error) (global.get $Stdlib_FormatErrorMessage::MESSAGE)))

  (func $Stdlib_FormatErrorMessage::get_remaining_items_label (param $num_items i32) (result i32)
    ;; Generate a "...X more errors" label string
    (local $instance i32)
    (local $offset i32)
    ;; Allocate a new dynamic string to contain the formatted output
    (local.tee $instance (call $Term::String::allocate_unsized))
    (local.set $offset (call $Term::String::get_offset (local.get $instance)))
    ;; Write the preceding ellipsis
    (@store-bytes $offset "...")
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the number of items
    (call $Utils::u32::write_string (local.get $num_items) (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    ;; Write the trailing label
    (@store-bytes $offset " more errors")
    (i32.add (local.get $offset))
    ;; Determine the total number of bytes written
    (i32.sub (call $Term::String::get_offset (local.get $instance)))
    (call $Term::String::init_unsized))

  (func $Stdlib_FormatErrorMessage::get_iterator_length (param $iterator i32) (param $state i32) (result i32 i32)
    (local $size_hint i32)
    (if (result i32 i32)
      ;; If the iterator provides a static size hint, return that
      (i32.ne
        (local.tee $size_hint (call $Term::traits::size_hint (local.get $iterator)))
        (global.get $NULL))
      (then
        (local.get $size_hint)
        (global.get $NULL))
      (else
        ;; Otherwise calculate the value dynamically
        (call $Stdlib_FormatErrorMessage::get_unsized_iterator_length (local.get $iterator) (local.get $state)))))

  (func $Stdlib_FormatErrorMessage::get_unsized_iterator_length (param $iterator i32) (param $state i32) (result i32 i32)
    (local $num_items i32)
    (local $item i32)
    (local $iterator_state i32)
    (local $dependencies i32)
    (local.set $dependencies (global.get $NULL))
    ;; Consume the iterator, counting the number of iterations
    (@iterate $LOOP $iterator $item $iterator_state $state $dependencies
      (local.set $num_items (i32.add (i32.const 1) (local.get $num_items))))
    (local.get $num_items)
    (local.get $dependencies))

  (func $Stdlib_FormatErrorMessage::combine_error_messages (param $messages i32) (param $state i32) (result i32 i32)
    (call $Term::String::traits::collect
      (call $Term::IntersperseIterator::new
        (local.get $messages)
        (global.get $Stdlib_FormatErrorMessage::NEWLINE))
      (local.get $state))))
