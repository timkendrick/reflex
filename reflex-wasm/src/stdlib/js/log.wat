;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Stdlib_Log::SPACE i32 (call $Term::String::from_char (@char " ")))
  (@const $Stdlib_Log::NEWLINE i32 (call $Term::String::from_char (@char "\n")))

  (@builtin $Stdlib_Log "Log"
    (@args (@eager $self) (@variadic (@eager $varargs)))

    (@impl
      (i32.or (i32.const 0xFFFFFFFF))
      (call $TermType::implements::iterate)
      (func $Stdlib_Log::impl::<any>::<iterate> (param $self i32) (param $varargs i32) (param $state i32) (result i32 i32)
        (local $output i32)
        (local $offset i32)
        ;; Resolve the arguments and convert to debug representation, combining them into a space-separated string
        (call $Term::String::traits::collect
          (call $Term::IntersperseIterator::new
            (call $Term::MapIterator::new
              (call $Term::FlattenIterator::new
                (call $Term::List::create_pair
                  (call $Term::OnceIterator::new (local.get $self))
                  (local.get $varargs)))
              (call $Term::Builtin::new (global.get $Stdlib_Debug)))
            (global.get $Stdlib_Log::SPACE))
          (local.get $state))
        ;; Drop the dependencies of the iteration
        ;; (this ensures that passing additional log arguments does not incur side-effects)
        (drop)
        ;; Log the string to stdout
        (call $Io::log_term (local.tee $output))
        ;; Discard the number of bytes written
        (drop)
        ;; Discard the temporary string
        (call $Term::String::drop (local.get $output))
        ;; For the overall method return value, return the first argument
        (local.get $self)
        (global.get $NULL)))

    (@default
      (func $Stdlib_Log::impl::default (param $self i32) (param $varargs i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Log)
            (call $Term::List::create_pair (local.get $self) (local.get $varargs))))
        (global.get $NULL)))))
