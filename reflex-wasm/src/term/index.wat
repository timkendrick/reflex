;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@include "./application.wat")
  (@include "./boolean.wat")
  (@include "./builtin.wat")
  (@include "./cell.wat")
  (@include "./collection/hashmap.wat")
  (@include "./collection/list.wat")
  (@include "./collection/record.wat")
  (@include "./collection/tree.wat")
  (@include "./condition.wat")
  (@include "./effect.wat")
  (@include "./float.wat")
  (@include "./int.wat")
  (@include "./iterator/chain.wat")
  (@include "./iterator/empty.wat")
  (@include "./iterator/evaluate.wat")
  (@include "./iterator/filter.wat")
  (@include "./iterator/flatten.wat")
  (@include "./iterator/hashmap_keys.wat")
  (@include "./iterator/hashmap_values.wat")
  (@include "./iterator/integers.wat")
  (@include "./iterator/map.wat")
  (@include "./iterator/once.wat")
  (@include "./iterator/range.wat")
  (@include "./iterator/repeat.wat")
  (@include "./iterator/skip.wat")
  (@include "./iterator/take.wat")
  (@include "./iterator/zip.wat")
  (@include "./nil.wat")
  (@include "./partial.wat")
  (@include "./pointer.wat")
  (@include "./signal.wat")
  (@include "./string.wat")
  (@include "./symbol.wat")

  ;; Declare term types
  (@let $typenames
    (@list
      "Application"
      "Partial"
      "Builtin"
      "Effect"
      "Signal"
      "Condition"
      "Nil"
      "Boolean"
      "Int"
      "Float"
      "Symbol"
      "String"
      "List"
      "Record"
      "Hashmap"
      "Tree"
      "EmptyIterator"
      "OnceIterator"
      "RepeatIterator"
      "SkipIterator"
      "TakeIterator"
      "ChainIterator"
      "ZipIterator"
      "MapIterator"
      "FilterIterator"
      "FlattenIterator"
      "EvaluateIterator"
      "IntegersIterator"
      "RangeIterator"
      "HashmapKeysIterator"
      "HashmapValuesIterator"
      "Cell"
      "Pointer")

    ;; Declare term type constants
    (@map $typename
      (@get $typenames)
      (global (@concat "$TermType::" (@get $typename)) i32 (i32.const (@get $_))))

    (func $TermType::startup
      ;; Allocate singleton instances
      ;; TODO: Generate startup snapshot via compile-time macros
      (@map $typename
        (@get $typenames)
        (call (@concat "$" (@get $typename) "::startup"))))

    (func $TermType::traits::is_static (param $self i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::is_static") (local.get $self)))))
        ;; Default implementation
        (global.get $TRUE)))

    (func $TermType::traits::is_atomic (param $self i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::is_atomic") (local.get $self)))))
        ;; Default implementation
        (global.get $TRUE)))

    (func $TermType::traits::is_truthy (param $self i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::is_truthy") (local.get $self)))))
        ;; Default implementation
        (global.get $TRUE)))

    (func $TermType::traits::hash (param $self i32) (param $state i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::hash") (local.get $self) (local.get $state)))))
        ;; Default implementation
        (global.get $NULL)))

    (func $TermType::traits::equals (param $self i32) (param $other i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::equals") (local.get $self) (local.get $other)))))
        ;; Default implementation
        (global.get $FALSE)))

    (func $TermType::traits::write_json (param $self i32) (param $offset i32) (result i32)
      (@branch
        ;; Delegate method to underlying term type implementations
        (call $Term::get_type (local.get $self))
        (@list
          (@map $typename
            (@get $typenames)
            (return (call (@concat "$" (@get $typename) "::traits::write_json") (local.get $self) (local.get $offset)))))
        ;; Default implementation
        (call $Term::traits::write_json (call $Record::empty) (local.get $offset))))

    ;; Trait implementations
    ;; TODO: Refactor manual trait delegation implementations into macro

    (@let $trait_typenames
      (@list
        "Application"
        "Effect"
        "Pointer")

      (func $TermType::implements::evaluate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $TermType::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$" (@get $typename) "::traits::evaluate") (local.get $self) (local.get $state))))))
          ;; Default implementation
          (local.get $self)
          (global.get $NULL))))

    (@let $trait_typenames
      (@list
        "Builtin"
        "Partial"
        "Signal")

      (func $TermType::implements::apply (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $TermType::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$" (@get $typename) "::traits::apply") (local.get $self) (local.get $args) (local.get $state))))))
          ;; Default implementation
          (call $Signal::of (call $Condition::invalid_function_target (local.get $self)))
          (global.get $NULL))))

    (@let $trait_typenames
      (@list
        "List"
        "Record"
        "Hashmap"
        "Tree"
        "EmptyIterator"
        "OnceIterator"
        "RepeatIterator"
        "SkipIterator"
        "TakeIterator"
        "ChainIterator"
        "ZipIterator"
        "MapIterator"
        "FilterIterator"
        "FlattenIterator"
        "EvaluateIterator"
        "IntegersIterator"
        "RangeIterator"
        "HashmapKeysIterator"
        "HashmapValuesIterator")

      (func $TermType::implements::iterate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $TermType::traits::iterate (param $self i32) (result i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$" (@get $typename) "::traits::iterate") (local.get $self))))))
          ;; Default implementation
          (global.get $NULL)))

      (func $TermType::implements::size_hint (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $TermType::traits::size_hint (param $self i32) (result i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$" (@get $typename) "::traits::size_hint") (local.get $self))))))
          ;; Default implementation
          (global.get $NULL)))

      (func $TermType::implements::next (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $TermType::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$" (@get $typename) "::traits::next") (local.get $self) (local.get $iterator_state) (local.get $state))))))
          ;; Default implementation
          (global.get $NULL)
          (global.get $NULL)
          (global.get $NULL))))))
