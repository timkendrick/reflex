;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (global $Term::NUM_HEADER_FIELDS i32 (i32.const 3))

  (func $Term::new (param $term_type i32) (param $num_fields i32) (result i32)
    (local $self i32)
    ;; Reserve space for the term header, followed by any additional struct fields
    ;; Each field occupies 4 bytes
    (local.tee $self (call $Allocator::allocate (i32.mul (i32.add (global.get $Term::NUM_HEADER_FIELDS) (local.get $num_fields)) (i32.const 4))))
    ;; Store the term type discriminant
    (call $Term::set_type (local.get $self) (local.get $term_type))
    (call $Term::set_num_fields (local.get $self) (local.get $num_fields)))

  (func $Term::init (param $self i32) (result i32)
    ;; Store the precomputed term hash
    (call $Term::set_hash (local.get $self) (call $Term::traits::hash (local.get $self) (call $Hash::new)))
    (local.get $self))

  (func $Term::drop (param $self i32)
    (call $Term::redirect (local.get $self) (call $Signal::invalid_pointer)))

  (func $Term::redirect (param $self i32) (param $target i32)
    (call $Term::set_type (local.get $self) (global.get $TermType::Pointer))
    (call $Term::set_hash (local.get $self) (call $Term::get_hash (local.get $target)))
    ;; This assumes that the previous term had at least one field (singleton instances should be created for zero-field terms)
    (call $Pointer::set_target (local.get $self) (local.get $target)))

  (func $Term::get_type (export "getTermType") (param $self i32) (result i32)
    ;; Retrieve the header field value from the correct offset
    (call $Term::get_header_field (local.get $self) (i32.const 0)))

  (func $Term::set_type (param $self i32) (param $value i32)
    ;; Update the header field value at the correct offset
    (call $Term::set_header_field (local.get $self) (i32.const 0) (local.get $value)))

  (func $Term::get_num_fields (param $self i32) (result i32)
    ;; Retrieve the header field value from the correct offset
    (call $Term::get_header_field (local.get $self) (i32.const 1)))

  (func $Term::set_num_fields (param $self i32) (param $value i32)
    ;; Update the header field value at the correct offset
    (call $Term::set_header_field (local.get $self) (i32.const 1) (local.get $value)))

  (func $Term::get_hash (export "getTermHash") (param $self i32) (result i32)
    ;; Retrieve the header field value from the correct offset
    (call $Term::get_header_field (local.get $self) (i32.const 2)))

  (func $Term::set_hash (param $self i32) (param $value i32)
    ;; Update the header field value at the correct offset
    (call $Term::set_header_field (local.get $self) (i32.const 2) (local.get $value)))

  (func $Term::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; First write the term type discriminant to the hash
    ;; (it's safe to hash this as a single byte because there are fewer than 255 term types)
    (local.set $state (call $Hash::write_byte (local.get $state) (call $Term::get_type (local.get $self))))
    ;; Then compute the hash according to the term type implementation
    (call $TermType::traits::hash (local.get $self) (local.get $state)))

  (func $Term::traits::equals (export "equals") (param $self i32) (param $other i32) (result i32)
    (local $self_type i32)
    (local $other_type i32)
    (if (result i32)
      ;; Compare term pointers
      (i32.eq (local.get $self) (local.get $other))
      (then
        ;; If term pointers match, this is the same term
        (global.get $TRUE))
      (else
        (if (result i32)
          ;; If the term types or term hashes differ, we can confidently say the terms are not equal
          (i32.or
            (i32.ne (call $Term::get_type (local.get $self)) (call $Term::get_type (local.get $other)))
            (i32.ne (call $Term::get_hash (local.get $self)) (call $Term::get_hash (local.get $other))))
          (then
            (global.get $FALSE))
          (else
            ;; Confirm equality according to the term type implementation
            (call $TermType::traits::equals (local.get $self) (local.get $other)))))))

  (func $Term::traits::clone (param $self i32) (result i32)
    (local $instance i32)
    (local $size i32)
    (memory.copy
      (local.tee $instance
        (call $Allocator::allocate
          (local.tee $size
            (i32.mul
              (i32.add (global.get $Term::NUM_HEADER_FIELDS) (call $Term::get_num_fields (local.get $self)))
              (i32.const 4)))))
      (local.get $self)
      (local.get $size))
    (local.get $instance))

  (func $Term::traits::is_static (param $self i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::is_static (local.get $self)))

  (func $Term::traits::is_atomic (param $self i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::is_atomic (local.get $self)))

  (func $Term::traits::is_truthy (param $self i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::is_truthy (local.get $self)))

  (func $Term::traits::write_json (param $self i32) (param $offset i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::write_json (local.get $self) (local.get $offset)))

  (func $Term::traits::evaluate (export "evaluate") (param $self i32) (param $state i32) (result i32 i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::evaluate (local.get $self) (local.get $state)))

  (func $Term::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::apply (local.get $self) (local.get $args) (local.get $state)))

  (func $Term::traits::iterate (param $self i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::iterate (local.get $self)))

  (func $Term::traits::size_hint (param $self i32) (result i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::size_hint (local.get $self)))

  (func $Term::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    ;; Delegate trait implementation to the term type implementation
    (call $TermType::traits::next (local.get $self) (local.get $iterator_state) (local.get $state)))

  (func $Term::get_header_field_pointer (param $self i32) (param $field_index i32) (result i32)
    ;; Each field is 4 bytes wide
    (i32.add (local.get $self) (i32.mul (local.get $field_index) (i32.const 4))))

  (func $Term::get_header_field (param $self i32) (param $field i32) (result i32)
    ;; Load the field value at the correct address
    (i32.load (call $Term::get_header_field_pointer (local.get $self) (local.get $field))))

  (func $Term::set_header_field (param $self i32) (param $field i32) (param $value i32)
    ;; Update the field value at the correct address
    (i32.store (call $Term::get_header_field_pointer (local.get $self) (local.get $field)) (local.get $value)))

  (func $Term::get_field_pointer (param $self i32) (param $field_index i32) (result i32)
    ;; Struct fields are stored immediately after the header fields
    (call $Term::get_header_field_pointer (local.get $self) (i32.add (global.get $Term::NUM_HEADER_FIELDS) (local.get $field_index))))

  (func $Term::get_field (param $self i32) (param $field_index i32) (result i32)
    ;; Load the field value at the correct address
    (i32.load (call $Term::get_field_pointer (local.get $self) (local.get $field_index))))

  (func $Term::set_field (param $self i32) (param $field_index i32) (param $value i32)
    ;; Update the field value at the correct address
    (i32.store (call $Term::get_field_pointer (local.get $self) (local.get $field_index)) (local.get $value)))

  (func $Term::get_f64_field (param $self i32) (param $field_index i32) (result f64)
    ;; Load the given field offset as a 64-bit float
    (f64.load (call $Term::get_field_pointer (local.get $self) (local.get $field_index))))

  (func $Term::set_f64_field (param $self i32) (param $field_index i32) (param $value f64)
    ;; Update the given field offset as a 64-bit float
    (f64.store (call $Term::get_field_pointer (local.get $self) (local.get $field_index)) (local.get $value)))

  (func $Term::to_json (export "toJson") (param $self i32) (result i32)
    (local $output i32)
    (call $String::init_unsized
      (local.tee $output (call $String::allocate_unsized))
      (i32.sub
        (call $Term::traits::write_json (local.get $self) (call $String::get_char_pointer (local.get $output) (i32.const 0)))
        (call $String::get_char_pointer (local.get $output) (i32.const 0))))))
