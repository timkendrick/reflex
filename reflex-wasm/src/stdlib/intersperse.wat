;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Intersperse "Intersperse"
    (@args (@strict $self) (@strict $separator))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Intersperse::impl::<iterate>::String (param $self i32) (param $separator i32) (param $state i32) (result i32 i32)
        (call $Term::IntersperseIterator::new (local.get $self) (local.get $separator))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Intersperse::impl::default (param $self i32) (param $separator i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Intersperse)
            (call $Term::List::create_pair (local.get $self) (local.get $separator))))
        (global.get $NULL)))))
