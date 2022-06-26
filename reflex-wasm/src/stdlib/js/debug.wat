;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Debug "Debug"
    (@args (@eager $self))

    (@default
      (func $Stdlib_Debug::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (local $instance i32)
        (local $end_offset i32)
        ;; Allocate a new dynamic string term
        (local.set $instance (call $Term::String::allocate_unsized))
        ;; Serialize the input term into the newly-allocated string contents
        (local.set $end_offset
          (call $Term::traits::debug
            (local.get $self)
            (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0))))
        ;; Initialize the dynamic string term
        (call $Term::String::init_unsized
          (local.get $instance)
          (i32.sub
            (local.get $end_offset)
            (call $Term::String::get_char_pointer (local.get $instance) (i32.const 0))))
        (global.get $NULL)))))
