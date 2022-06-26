;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_ToString "ToString"
    (@args (@strict $self))

    (@impl
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_ToString::impl::String (param $self i32) (param $state i32) (result i32 i32)
        (local.get $self)
        (global.get $NULL)))

    (@default
      (func $Stdlib_ToString::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local $output i32)
        (local $offset i32)
        ;; Allocate a new dynamic string term
        (local.set $output (call $Term::String::allocate_unsized))
        ;; Serialize the input term into the newly-allocated string contents
        (local.set $offset
          (call $Term::traits::display
            (local.get $self)
            (call $Term::String::get_char_pointer (local.get $output) (i32.const 0))))
        ;; Initialize the dynamic string term
        (call $Term::String::init_unsized
          (local.get $output)
          (i32.sub
            (local.get $offset)
            (call $Term::String::get_char_pointer (local.get $output) (i32.const 0))))
        (global.get $NULL)))))
