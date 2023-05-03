;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Construct "Construct"
    (@args (@strict $self) (@variadic (@strict $arg_list)))

    (@impl
      (i32.eq (global.get $TermType::Constructor))
      (call $TermType::implements::iterate)
      (func $Stdlib_Construct::impl::Constructor::<iterate> (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $properties i32)
        (local $dependencies i32)
        ;; Consume the first argument from the argument list
        (call $Term::traits::next (local.get $arg_list) (global.get $NULL) (local.get $state))
        ;; Update the dependencies with the iterator dependencies
        (local.set $dependencies)
        ;; Ignore the iterator state as we only care about the first variadic argument
        (drop)
        ;; If a properties argument was provided, attempt to parse it into a newly-constructed record based on the type of the argument
        (if (result i32 i32)
          (i32.ne (local.tee $properties) (global.get $NULL))
          (then
            (@switch
              (@list
                (@list
                  (call $Term::Record::is (local.get $properties))
                  (block
                    ;; Invoke the underlying implementation
                    (call $Stdlib_Construct::impl::Constructor::<iterate>::Record
                      (local.get $self)
                      (local.get $arg_list)
                      (local.get $properties)
                      (local.get $state))
                    ;; Combine the dependencies with the accumulated dependencies
                    (call $Dependencies::traits::union (local.get $dependencies))
                    (return)))
                (@list
                  (call $Term::implements::iterate (local.get $properties))
                  (block
                    ;; Invoke the underlying implementation
                    (call $Stdlib_Construct::impl::Constructor::<iterate>::<iterate>
                      (local.get $self)
                      (local.get $arg_list)
                      (local.get $properties)
                      (local.get $state))
                    ;; Combine the dependencies with the accumulated dependencies
                    (call $Dependencies::traits::union (local.get $dependencies))
                    (return))))
              ;; Otherwise if the properties argument is an unknown type, return an error
              (call $Stdlib_Construct::impl::default (local.get $self) (local.get $arg_list) (local.get $state))
              ;; Combine the error dependencies with the accumulated dependencies
              (call $Dependencies::traits::union (local.get $dependencies))))
          (else
            ;; If no properties argument was provided, return an error
            (call $Stdlib_Construct::impl::default (local.get $self) (local.get $arg_list) (local.get $state))
            ;; Combine the error dependencies with the accumulated dependencies
            (call $Dependencies::traits::union (local.get $dependencies))))))

    (@impl
      (call $TermType::implements::apply)
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_Construct::impl::<apply>::List (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::traits::apply (local.get $self) (local.get $arg_list) (local.get $state))))

    (@impl
      (call $TermType::implements::apply)
      (call $TermType::implements::iterate)
      (func $Stdlib_Construct::impl::<apply>::<iterate> (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (local $dependencies i32)
        (call $Term::List::traits::collect (local.get $arg_list) (local.get $state))
        (local.set $dependencies)
        (local.set $arg_list)
        (call $Stdlib_Construct::impl::<apply>::List (local.get $self) (local.get $arg_list) (local.get $state))
        (call $Dependencies::traits::union (local.get $dependencies))))

    (@default
      (func $Stdlib_Construct::impl::default (param $self i32) (param $arg_list i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Construct)
            (call $Term::List::create_pair (local.get $self) (local.get $arg_list))))
        (global.get $NULL))))

  (func $Stdlib_Construct::impl::Constructor::<iterate>::Record (param $self i32) (param $arg_list i32) (param $properties i32) (param $state i32) (result i32 i32)
    (local $instance i32)
    (if (result i32 i32)
      ;; If the properties record can be parsed into a record with the correct shape, return that
      (i32.ne
        (local.tee $instance (call $Term::Constructor::parse_record (local.get $self) (local.get $properties)))
        (global.get $NULL))
      (then
        (local.get $instance)
        (global.get $NULL))
      (else
        ;; Otherwise return an error
        (call $Stdlib_Construct::impl::default (local.get $self) (local.get $arg_list) (local.get $state)))))

  (func $Stdlib_Construct::impl::Constructor::<iterate>::<iterate> (param $self i32) (param $arg_list i32) (param $properties i32) (param $state i32) (result i32 i32)
    (local $keys i32)
    (local $values i32)
    (local $dependencies i32)
    ;; Collect the list of property values
    (call $Term::List::traits::collect (local.get $properties) (local.get $state))
    ;; Pop the iteration dependencies off the stack, leaving the list of values on top of the stack
    (local.set $dependencies)
    (if (result i32 i32)
      (i32.eq
        (call $Term::List::get_length (local.tee $values))
        (call $Term::List::get_length (local.tee $keys (call $Term::Constructor::get::keys (local.get $self)))))
      (then
        ;; Construct a new record from the keys and values
        (call $Term::Record::new (local.get $keys) (local.get $values))
        ;; Push the accumulated dependencies back onto the stack
        (local.get $dependencies))
      (else
        ;; Otherwise return an error
        (call $Stdlib_Construct::impl::default (local.get $self) (local.get $arg_list) (local.get $state))
        ;; Combine the error dependencies with the accumulated dependencies
        (call $Dependencies::traits::union (local.get $dependencies))))))
