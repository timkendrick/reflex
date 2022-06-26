;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@builtin $Stdlib_Skip
    (@args (@strict $self) (@strict $count))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Skip::impl::<iterate>::Int (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (call $Term::SkipIterator::new (local.get $self) (call $Term::Int::get::value (local.get $count)))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Skip::impl::<iterate>::Float (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (local $count_value i32)
        (if (result i32 i32)
          (i32.ne (local.tee $count_value (call $Term::Float::get_non_negative_integer_value (local.get $count))) (global.get $NULL))
          (then
            (call $Term::SkipIterator::new (local.get $self) (local.get $count_value))
            (global.get $NULL))
          (else
            (call $Term::Signal::of
              (call $Term::Condition::invalid_builtin_function_args
                (global.get $Stdlib_Skip)
                (call $Term::List::create_pair (local.get $self) (local.get $count))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Skip::impl::default (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Skip)
            (call $Term::List::create_pair (local.get $self) (local.get $count))))
        (global.get $NULL)))))
