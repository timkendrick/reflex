;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@method $Stdlib_Take
    (@args (@strict $self) (@strict $count))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Take::impl::<iterate>::Int (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (call $TakeIterator::new (local.get $self) (call $Int::get::value (local.get $count)))
        (global.get $NULL)))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Take::impl::<iterate>::Float (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (local $index i32)
        (if (result i32 i32)
          (i32.ne (local.tee $index (call $Float::get_non_negative_integer_value (local.get $count))) (global.get $NULL))
          (then
            (call $TakeIterator::new (local.get $self) (call $Int::get::value (local.get $count)))
            (global.get $NULL))
          (else
            (call $Signal::of
              (call $Condition::invalid_builtin_function_args
                (global.get $Stdlib_Take)
                (call $List::create_pair (local.get $self) (local.get $count))))
            (global.get $NULL)))))

    (@default
      (func $Stdlib_Take::impl::default (param $self i32) (param $count i32) (param $state i32) (result i32 i32)
        (call $Signal::of
          (call $Condition::invalid_builtin_function_args
            (global.get $Stdlib_Take)
            (call $List::create_pair (local.get $self) (local.get $count))))
        (global.get $NULL)))))
