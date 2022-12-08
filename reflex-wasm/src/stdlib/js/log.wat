;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const $Stdlib_Log::SPACE i32 (call $Term::String::from_char (@char " ")))
  (@const $Stdlib_Log::NEWLINE i32 (call $Term::String::from_char (@char "\n")))

  (@builtin $Stdlib_Log "Log"
    (@args (@strict $self))

    (@impl
      (call $TermType::implements::iterate)
      (func $Stdlib_Log::impl::<iterate> (param $self i32) (param $state i32) (result i32 i32)
        (local $output i32)
        (local $dependencies i32)
        (local $offset i32)
        ;; Resolve the arguments and convert to debug representation, combining them into a space-separated string
        (call $Term::String::traits::collect
          (call $Term::IntersperseIterator::new
            (call $Term::EvaluateIterator::new
              (call $Term::MapIterator::new
                (local.get $self)
                (call $Term::Builtin::new (global.get $Stdlib_Debug))))
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
        ;; For the overall method return value, consume the first item from the argument iterator
        (call $Term::traits::next (local.get $self) (global.get $NULL) (local.get $state))
        (local.set $dependencies)
        ;; Drop the iterator state, as we only need the first item
        (drop)
        (local.get $dependencies)))

    (@default
      (func $Stdlib_Log::impl::default (param $self i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Log)
            (call $Term::List::of (local.get $self))))
        (global.get $NULL)))))
