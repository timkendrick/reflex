;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_StringifyJson
    (@args (@strict $self))

    (@impl
      (call $TermType::implements::to_json)
      (func $Stdlib_StringifyJson::impl::<to_json> (param $self i32) (param $state i32) (result i32 i32)
        (local $output i32)
        (local $offset i32)
        (local.set $output (call $Term::String::allocate_unsized))
        (local.set $offset
          (call $Term::traits::to_json
            (local.get $self)
            (call $Term::String::get_char_pointer (local.get $output) (i32.const 0))))
        ;; Determine whether a non-serializable term was encountered
        (if (result i32 i32)
          (i32.ne (global.get $TRUE))
          (then
            ;; If a non-serializable term was encountered, dispose of the temporary dynamic string
            ;; FIXME: improve dynamic string deallocation
            ;; Reset the heap space occupied by the temporary dynamic string with zeroes
            (memory.fill
              (local.get $output)
              (i32.const 0)
              (i32.sub (call $Allocator::get_offset) (local.get $output)))
            ;; Reset the allocator heap offset
            (call $Allocator::set_offset (local.get $output))
            ;; Return the default error signal
            (call $Stdlib_StringifyJson::impl::default (local.get $self) (local.get $state)))
          (else
            ;; Initialize the dynamic string term
            (call $Term::String::init
              (local.get $output)
              (i32.sub
                (local.get $offset)
                (call $Term::String::get_char_pointer (local.get $output) (i32.const 0))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_StringifyJson::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_StringifyJson)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
