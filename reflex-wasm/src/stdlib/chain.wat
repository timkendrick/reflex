;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Chain "Chain"
    (@args (@strict $self) (@strict $other))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Chain::impl::List::List (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (local $self_length i32)
        (local $other_length i32)
        (local $instance i32)
        ;; If either of the two input lists is empty, return the other one
        (if (result i32 i32)
          (i32.or
            (i32.eqz (local.tee $self_length (call $Term::List::get_length (local.get $self))))
            (i32.eqz (local.tee $other_length (call $Term::List::get_length (local.get $other)))))
          (then
            (select
              (local.get $other)
              (local.get $self)
              (i32.eqz (local.get $self_length)))
            (global.get $NULL))
          (else
            ;; Otherwise allocate a new list term with enough space for the contents of both lists
            (local.tee $instance (call $Term::List::allocate (i32.add (local.get $self_length) (local.get $other_length))))
            ;; Copy the contents of the first list into the newly-allocated list term
            (memory.copy
              (call $Term::List::get::items::pointer (local.get $instance) (i32.const 0))
              (call $Term::List::get_items (local.get $self))
              (i32.mul (i32.const 4) (local.get $self_length)))
            ;; Copy the contents of the second list into the newly-allocated list term
            (memory.copy
              (call $Term::List::get::items::pointer (local.get $instance) (local.get $self_length))
              (call $Term::List::get_items (local.get $other))
              (i32.mul (i32.const 4) (local.get $other_length)))
            ;; Initialize the newly-allocated list term
            (call $Term::List::init (i32.add (local.get $self_length) (local.get $other_length)))
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::iterate)
      (func $Stdlib_Chain::impl::<iterate>::<iterate> (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::FlattenIterator::new
          (call $Term::List::create_pair (local.get $self) (local.get $other)))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Chain::impl::default (param $self i32) (param $other i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Chain)
            (call $Term::List::create_pair (local.get $self) (local.get $other))))
        (global.get $NULL)))))
