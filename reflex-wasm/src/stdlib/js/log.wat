;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Stdlib_Log::NEWLINE i32 (call $Term::String::from_char (@char "\n")))

  (@builtin $Stdlib_Log "Log"
    (@args (@strict $self))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_Log::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $args i32)
        (local $arg i32)
        (local $iterator_state i32)
        (local $dependencies i32)
        (local $result i32)
        (local $offset i32)
        (local $length i32)
        (local.set $result (global.get $NULL))
        (local.set $dependencies (global.get $NULL))
        (local.set $args
          (call $Term::MapIterator::new
            (local.get $self)
            (call $Term::Builtin::new (global.get $Stdlib_ResolveDeep))))
        (@iterate $args $arg $iterator_state $state $dependencies
          ;; Evaluate the argument
          (call $Term::traits::evaluate (local.get $arg) (local.get $state))
          (local.set $dependencies (call $Dependencies::traits::union (local.get $dependencies)))
          ;; Write the serialized argument at the end of the currently-allocated memory
          (call $Term::traits::display
            (local.tee $arg)
            (local.tee $offset (call $Allocator::get_offset)))
          (local.set $length (i32.sub (local.get $offset)))
          ;; If this is the first argument, store it to be used as the overall return value
          (if
            (i32.eq (global.get $NULL) (local.get $result))
            (then
              (local.set $result (local.get $arg))))
          ;; Write the serialized argument to stdout
          (call $Io::write_stdout (local.get $offset) (local.get $length))
          ;; Discard the number of bytes written
          (drop)
          ;; Dispose the temporarily-allocated memory
          (call $Allocator::shrink
            (i32.add (local.get $offset) (local.get $length))
            (local.get $length)))
        ;; Write a newline to stdout
        (call $Io::write_stdout
          (call $Term::String::get_offset (global.get $Stdlib_Log::NEWLINE))
          (call $Term::String::get_length (global.get $Stdlib_Log::NEWLINE)))
        ;; Discard the number of bytes written
        (drop)
        ;; If no arguments were passed, return an error
        (if (result i32 i32)
          (i32.eq (global.get $NULL) (local.get $result))
          (then
            (call $Stdlib_Log::impl::default (local.get $self) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies)))
          ;; Otherwise return the first argument
          (else
            (local.get $result)
            (local.get $dependencies)))))

    (@default
      (func $Stdlib_Log::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Log)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
