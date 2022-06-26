;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_ToRequest::FIELD_URL "url")
  (@const-string $Stdlib_ToRequest::FIELD_METHOD "method")
  (@const-string $Stdlib_ToRequest::FIELD_HEADERS "headers")
  (@const-string $Stdlib_ToRequest::FIELD_BODY "body")

  (@const $Stdlib_ToRequest::FIELDS i32
    (@depends-on $Stdlib_ToRequest::FIELD_URL)
    (@depends-on $Stdlib_ToRequest::FIELD_METHOD)
    (@depends-on $Stdlib_ToRequest::FIELD_HEADERS)
    (@depends-on $Stdlib_ToRequest::FIELD_BODY)
    (func (result i32)
      (local $instance i32)
      (local.tee $instance (call $Term::List::allocate (i32.const 4)))
      (call $Term::List::set_item (local.get $instance) (i32.const 0) (global.get $Stdlib_ToRequest::FIELD_URL))
      (call $Term::List::set_item (local.get $instance) (i32.const 1) (global.get $Stdlib_ToRequest::FIELD_METHOD))
      (call $Term::List::set_item (local.get $instance) (i32.const 2) (global.get $Stdlib_ToRequest::FIELD_HEADERS))
      (call $Term::List::set_item (local.get $instance) (i32.const 3) (global.get $Stdlib_ToRequest::FIELD_BODY))
      (call $Term::List::init (i32.const 4))))

  (@const-string $Stdlib_ToRequest::METHOD_GET "GET")

  (@const $Stdlib_ToRequest::CONSTRUCTOR i32
    (@depends-on $Stdlib_ToRequest::FIELDS)
    (call $Term::Constructor::new (global.get $Stdlib_ToRequest::FIELDS)))

  (@builtin $Stdlib_ToRequest
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ToRequest::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local $values i32)
        ;; Push the constructor keys onto the stack (used to construct the record later)
        (global.get $Stdlib_ToRequest::FIELDS)
        ;; Allocate a new list to hold the record values
        (local.tee $values (call $Term::List::allocate (i32.const 4)))
        ;; Store the provided URL in the url field
        (call $Term::List::set_item (local.get $values) (i32.const 0) (local.get $self))
        ;; Store the default method string (GET) in the method field
        (call $Term::List::set_item (local.get $values) (i32.const 1) (global.get $Stdlib_ToRequest::METHOD_GET))
        ;; Store an empty record in the headers field
        (call $Term::List::set_item (local.get $values) (i32.const 2) (call $Term::Record::empty))
        ;; Store an empty value in the body field
        (call $Term::List::set_item (local.get $values) (i32.const 3) (call $Term::Nil::new))
        ;; Instantiate the list of record values
        (call $Term::List::init (i32.const 4))
        ;; Create a record from the keys and values lists currently at the top of the stack
        (call $Term::Record::new)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ToRequest::impl::Record (param $self i32) (param $state i32) (result i32 i32)
        (local $instance i32)
        (if (result i32 i32)
          ;; If the record can be parsed into an instance of the request constructor type, return the instance
          (i32.ne
            (local.tee $instance
              (call $Term::Constructor::parse_record
                (global.get $Stdlib_ToRequest::CONSTRUCTOR)
                (local.get $self)))
            (global.get $NULL))
          (then
            (local.get $instance)
            (global.get $NULL))
          (else
            ;; Otherwise return an error
            (call $Stdlib_ToRequest::impl::default (local.get $self) (local.get $state))))))

    (@default
      (func $Stdlib_ToRequest::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ToRequest)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
