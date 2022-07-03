;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@export $Term
    (@struct $Term
      (@field $hash i32)
      (@field $value (@import $TermType "./term_type/index.wat"))))

  ;; Declare term type wrapper methods
  (@let $TermType
    (@import $TermType "./term_type/index.wat")

    (@map $typename
      (@union_variants (@get $TermType))
      (@block

        (func (@concat "$Term::" (@get $typename) "::is") (param $self i32) (result i32)
          (i32.eq (global.get (@concat "$TermType::" (@get $typename))) (call $Term::get_type (local.get $self))))))

    (@delegate $Term (@get $TermType) (call $Term::pointer::value (local.get 0)))
    (@map $typename
      (@union_variants (@get $TermType))
      (@delegate
        $Term
        (@union_variant (@get $TermType) (@get $_))
        (call $Term::TermType::get::value (local.get 0))))

    (@map $signature
      (@signatures (@get $TermType))
      (@let $typename (@list_item (@get $signature) 0)
        (@let $args (@list_item (@get $signature) 1)
          (@block

          (func (@concat "$Term::" (@get $typename) "::new")(@map $arg (@get $args) (@block (param (@list_item (@get $arg) 0) (@list_item (@get $arg) 1)))) (result i32)
            (local i32)
            (local.tee (@length (@get $args)) (call $Term::allocate))
            (call (@concat "$" (@get $typename) "::construct") (call $Term::pointer::value (local.get (@length (@get $args))))(@map $arg (@get $args) (@block (local.get (@list_item (@get $arg) 0)))))
            (call $Term::init))))))

    (func $Term::traits::is_atomic (param $self i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (return (call (@concat "$Term::" (@get $typename) "::traits::is_atomic") (local.get $self)))))
        ;; Default implementation
        (global.get $TRUE)))

    (func $Term::traits::is_truthy (param $self i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (return (call (@concat "$Term::" (@get $typename) "::traits::is_truthy") (local.get $self)))))
        ;; Default implementation
        (global.get $TRUE)))

    (func $Term::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (return (call (@concat "$Term::" (@get $typename) "::traits::substitute") (local.get $self) (local.get $variables) (local.get $scope_offset)))))
        ;; Default implementation
        (global.get $NULL))))

  (export "display" (func $Term::traits::display))
  (export "debug" (func $Term::traits::debug))
  (export "evaluate" (func $Term::traits::evaluate))

  (func $Term::traits::equals (export "equals") (param $self i32) (param $other i32) (result i32)
    (if (result i32)
      ;; Compare term pointers
      (i32.eq (local.get $self) (local.get $other))
      (then
        ;; If term pointers match, this is the same term
        (global.get $TRUE))
      (else
        (if (result i32)
          ;; If the hashes differ, we can confidently say the terms are not equal
          (i32.ne
            (call $Term::get::hash (local.get $self))
            (call $Term::get::hash (local.get $other)))
          (then
            (global.get $FALSE))
          (else
            ;; Confirm equality according to the underlying term type implementation
            (call $TermType::traits::equals
              (call $Term::pointer::value (local.get $self))
              (call $Term::pointer::value (local.get $other))))))))

  (func $Term::traits::hash (param $self i32) (param $state i32) (result i32)
    ;; Compute the hash according to the term type implementation
    (call $TermType::traits::hash (call $Term::pointer::value (local.get $self)) (local.get $state)))

  (func $Term::traits::size (param $self i32) (result i32)
    (i32.add
      ;; Add 4 bytes for the term hash
      (i32.const 4)
      ;; Add the size of the underlying term type
      (call $TermType::traits::size (call $Term::pointer::value (local.get $self)))))

  (func $Term::allocate (result i32)
    (call $Allocator::allocate (call $Term::sizeof)))

  (func $Term::init (param $self i32) (result i32)
    ;; Precompute the term hash and store it for fast equality checks
    (call $Term::set::hash
      (local.get $self)
      (call $Term::traits::hash (local.get $self) (call $Hash::new)))
    (local.get $self))

  (func $Term::get_hash (export "getTermHash") (param $self i32) (result i32)
    (call $Term::get::hash (local.get $self)))

  (func $Term::get_type (export "getTermType") (param $self i32) (result i32)
    (call $Term::TermType::get::type (local.get $self)))

  (func $Term::get_value (param $self i32) (result i32)
    (call $Term::TermType::get::value (local.get $self)))

  (func $Term::is_static (param $self i32) (result i32)
    (i32.eqz (call $Term::implements::evaluate (local.get $self))))

  (func $Term::drop (param $self i32)
    (local $size i32)
    (local $end_offset i32)
    (local $foo i32)
    ;; If this was the most recently allocated object, nothing can ever have referenced it, so it can be safely wiped.
    ;; Otherwise there may theoretically still be dangling references to the disposed term, so we point it to an error.
    (if
      ;; Consult the current allocator offset to check whether this was the most recently allocated object
      (i32.eq
        (call $Allocator::get_offset)
        (local.tee $end_offset
          (i32.add
            (local.get $self)
            (local.tee $size (call $Term::traits::size (local.get $self))))))
      (then
        ;; If this was the most recently allocated object, wipe the memory
        (call $Allocator::shrink (local.get $end_offset) (local.get $size)))
      (else
        ;; Otherwise create a redirect from the old address to an 'invalid pointer' error
        (call $Term::redirect (local.get $self) (call $Term::Signal::invalid_pointer)))))

  (func $Term::redirect (param $self i32) (param $target i32)
    ;; TODO: When overwriting cell with redirect pointer, somehow mark truncated fields as elibible for GC
    (call $Term::set::hash (local.get $self) (call $Term::get::hash (local.get $target)))
    ;; This assumes that the previous term had at least one field (singleton instances should be created for zero-field terms)
    (call $Term::Pointer::set::target (call $Term::pointer::value (local.get $self)) (local.get $target)))

  (func $Term::traits::clone (param $self i32) (result i32)
    (local $instance i32)
    (local $size i32)
    (memory.copy
      (local.tee $instance
        (call $Allocator::allocate
          (call $Utils::i32::max_u
            (local.tee $size (call $Term::traits::size (local.get $self)))
            (call $Term::sizeof))))
      (local.get $self)
      (local.get $size))
    (local.get $instance)))
