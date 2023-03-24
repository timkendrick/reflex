;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Filter "Filter"
    (@args (@strict $self) (@strict $predicate))

    (@impl
      (call $TermType::implements::iterate)
      (call $TermType::implements::apply)
      (func $Stdlib_Filter::impl::<iterate>::<iterate> (param $self i32) (param $predicate i32) (param $state i32) (result i32 i32)
        (call $Term::FilterIterator::new (local.get $self) (local.get $predicate))
        (global.get $NULL)))

    (@default
      (func $Stdlib_Filter::impl::default (param $self i32) (param $predicate i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Filter)
            (call $Term::List::create_pair (local.get $self) (local.get $predicate))))
        (global.get $NULL)))))
