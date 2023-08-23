;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@include "./application.wat")
  (@include "./boolean.wat")
  (@include "./builtin.wat")
  (@include "./cell.wat")
  (@include "./condition.wat")
  (@include "./constructor.wat")
  (@include "./dependency.wat")
  (@include "./effect.wat")
  (@include "./float.wat")
  (@include "./hashmap.wat")
  (@include "./hashset.wat")
  (@include "./int.wat")
  (@include "./iterator/empty.wat")
  (@include "./iterator/evaluate.wat")
  (@include "./iterator/filter.wat")
  (@include "./iterator/flatten.wat")
  (@include "./iterator/hashmap_keys.wat")
  (@include "./iterator/hashmap_values.wat")
  (@include "./iterator/indexed_accessor.wat")
  (@include "./iterator/integers.wat")
  (@include "./iterator/intersperse.wat")
  (@include "./iterator/map.wat")
  (@include "./iterator/once.wat")
  (@include "./iterator/range.wat")
  (@include "./iterator/repeat.wat")
  (@include "./iterator/skip.wat")
  (@include "./iterator/take.wat")
  (@include "./iterator/zip.wat")
  (@include "./lambda.wat")
  (@include "./lazy_result.wat")
  (@include "./let.wat")
  (@include "./list.wat")
  (@include "./nil.wat")
  (@include "./partial.wat")
  (@include "./pointer.wat")
  (@include "./record.wat")
  (@include "./signal.wat")
  (@include "./string.wat")
  (@include "./symbol.wat")
  (@include "./timestamp.wat")
  (@include "./tree.wat")
  (@include "./variable.wat")

  (@let $TermType
    (@union $TermType
      (@import $Application "./application.wat")
      (@import $Boolean "./boolean.wat")
      (@import $Builtin "./builtin.wat")
      (@import $Cell "./cell.wat")
      (@import $Condition "./condition.wat")
      (@import $Constructor "./constructor.wat")
      (@import $Dependency "./dependency.wat")
      (@import $Effect "./effect.wat")
      (@import $Float "./float.wat")
      (@import $Hashmap "./hashmap.wat")
      (@import $Hashset "./hashset.wat")
      (@import $Int "./int.wat")
      (@import $Lambda "./lambda.wat")
      (@import $LazyResult "./lazy_result.wat")
      (@import $Let "./let.wat")
      (@import $List "./list.wat")
      (@import $Nil "./nil.wat")
      (@import $Partial "./partial.wat")
      (@import $Pointer "./pointer.wat")
      (@import $Record "./record.wat")
      (@import $Signal "./signal.wat")
      (@import $String "./string.wat")
      (@import $Symbol "./symbol.wat")
      (@import $Timestamp "./timestamp.wat")
      (@import $Tree "./tree.wat")
      (@import $Variable "./variable.wat")
      (@import $EmptyIterator "./iterator/empty.wat")
      (@import $EvaluateIterator "./iterator/evaluate.wat")
      (@import $FilterIterator "./iterator/filter.wat")
      (@import $FlattenIterator "./iterator/flatten.wat")
      (@import $HashmapKeysIterator "./iterator/hashmap_keys.wat")
      (@import $HashmapValuesIterator "./iterator/hashmap_values.wat")
      (@import $IndexedAccessorIterator "./iterator/indexed_accessor.wat")
      (@import $IntegersIterator "./iterator/integers.wat")
      (@import $IntersperseIterator "./iterator/intersperse.wat")
      (@import $MapIterator "./iterator/map.wat")
      (@import $OnceIterator "./iterator/once.wat")
      (@import $RangeIterator "./iterator/range.wat")
      (@import $RepeatIterator "./iterator/repeat.wat")
      (@import $SkipIterator "./iterator/skip.wat")
      (@import $TakeIterator "./iterator/take.wat")
      (@import $ZipIterator "./iterator/zip.wat"))

    (@derive $equals (@get $TermType))
    (@derive $hash (@get $TermType))

    (@export $TermType (@get $TermType))

    ;; Declare global term type constants
    (@map $typename
      (@union_variants (@get $TermType))
      (@block
        (global (@concat "$TermType::" (@get $typename)) (export (@concat "\"" "TermType_" (@get $typename) "\"")) i32 (i32.const (@get $_)))))

    (@map $signature
      (@signatures (@get $TermType))
      (@let $variant (@list_item (@get $signature) 1)
        (@block

          (func (@concat "$TermType::" (@get $variant) "::sizeof") (result i32)
            (i32.add
              ;; Add 4 bytes for the discriminant
              (i32.const 4)
              ;; Add the size of the underlying term type
              (call (@concat "$" (@get $variant) "::sizeof")))))))

    (func $TermType::traits::size (param $self i32) (result i32)
      (@branch
        ;; Determine term size according to the underlying term type implementation
        (call $TermType::get::type (local.get $self))
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (return
              (i32.add
                (i32.const 4)
                (call (@concat "$" (@get $typename) "::traits::size") (call $TermType::get::value (local.get $self)))))))
        ;; Default implementation
        (call $TermType::sizeof)))

    (func $TermType::traits::display (param $self i32) (param $offset i32) (result i32)
      (@branch
        ;; Format term type according to the underlying term type implementation
        (local.get $self)
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (block
              (@store-bytes $offset (@to-string (@get $typename)))
              (return (i32.add (local.get $offset))))))
        ;; Default implementation
        (local.get $offset)))

    (func $Term::traits::display (param $self i32) (param $offset i32) (result i32)
      (@branch
        ;; Format term according to the underlying term type implementation
        (call $Term::TermType::get::type (local.get $self))
        (@list
          (@map $typename
            (@union_variants (@get $TermType))
            (return (call (@concat "$Term::" (@get $typename) "::traits::display") (local.get $self) (local.get $offset)))))
        ;; Default implementation
        (local.get $offset)))

    (func $Term::traits::debug (param $self i32) (param $offset i32) (result i32)
      (if (result i32)
        (i32.eq (local.get $self) (global.get $NULL))
        (then
          (@store-bytes $offset "NULL")
          (i32.add (local.get $offset)))
        (else
          (@branch
            ;; Format term according to the underlying term type implementation
            (call $Term::TermType::get::type (local.get $self))
            (@list
              (@map $typename
                (@union_variants (@get $TermType))
                (return (call (@concat "$Term::" (@get $typename) "::traits::debug") (local.get $self) (local.get $offset)))))
            ;; Default implementation
            (local.get $offset)))))

    ;; Trait implementations
    ;; TODO: Refactor manual trait delegation implementations into macro
    (@let $trait_typenames
      (@list
        $Application
        $Effect
        $LazyResult
        $Let)

      (func $TermType::implements::evaluate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::evaluate (param $self i32) (result i32)
        (call $TermType::implements::evaluate (call $Term::get_type (local.get $self))))

      (func $Term::traits::evaluate (param $self i32) (param $state i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::evaluate") (local.get $self) (local.get $state))))))
          ;; Default implementation
          (local.get $self)
          (global.get $NULL))))

    (@let $trait_typenames
      (@list
        $Builtin
        $Partial
        $Constructor
        $Lambda
        $Signal)

      (func $TermType::implements::apply (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::apply (param $self i32) (result i32)
        (call $TermType::implements::apply (call $Term::get_type (local.get $self))))

      (func $Term::traits::arity (export "arity") (param $self i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::arity") (local.get $self))))))
          ;; Default implementation
          (i32.const 0)
          (global.get $FALSE)))

      (func $Term::traits::apply (export "apply") (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::apply") (local.get $self) (local.get $args) (local.get $state))))))
          ;; Default implementation
          (call $Term::Signal::of (call $Term::Condition::invalid_function_target (local.get $self)))
          (global.get $NULL))))

    (@let $trait_typenames
      (@list
        $List
        $Record
        $Hashmap
        $Hashset
        $Tree
        $EmptyIterator
        $EvaluateIterator
        $FilterIterator
        $FlattenIterator
        $HashmapKeysIterator
        $HashmapValuesIterator
        $IndexedAccessorIterator
        $IntegersIterator
        $IntersperseIterator
        $MapIterator
        $OnceIterator
        $RangeIterator
        $RepeatIterator
        $SkipIterator
        $TakeIterator
        $ZipIterator)

      (func $TermType::implements::iterate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::iterate (param $self i32) (result i32)
        (call $TermType::implements::iterate (call $Term::get_type (local.get $self))))

      (func $Term::traits::iterate (param $self i32) (result i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::iterate") (local.get $self))))))
          ;; Default implementation
          (global.get $NULL)))

      (func $TermType::implements::size_hint (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::size_hint (param $self i32) (result i32)
        (call $TermType::implements::size_hint (call $Term::get_type (local.get $self))))

      (func $Term::traits::size_hint (param $self i32) (result i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::size_hint") (local.get $self))))))
          ;; Default implementation
          (global.get $NULL)))

      (func $TermType::implements::next (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::next (param $self i32) (result i32)
        (call $TermType::implements::next (call $Term::get_type (local.get $self))))

      (func $Term::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::next") (local.get $self) (local.get $iterator_state) (local.get $state))))))
          ;; Default implementation
          (global.get $NULL)
          (global.get $NULL)
          (global.get $NULL))))

    (@let $trait_typenames
      (@list
        $Nil
        $Boolean
        $Int
        $Float
        $String
        $List
        $Record
        $Timestamp)

      (func $TermType::implements::to_json (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::implements::to_json (param $self i32) (result i32)
        (call $TermType::implements::to_json (call $Term::get_type (local.get $self))))

      (func $Term::traits::to_json (param $self i32) (param $offset i32) (result i32 i32)
        (local $self_type i32)
        (local.set $self_type (call $Term::get_type (local.get $self)))
        (@switch
          ;; Delegate method to underlying term type implementations
          (@list
            (@map $typename
              (@get $trait_typenames)
              (@list
                (i32.eq (local.get $self_type) (global.get (@concat "$TermType::" (@get $typename))))
                (return (call (@concat "$Term::" (@get $typename) "::traits::to_json") (local.get $self) (local.get $offset))))))
          ;; Default implementation
          (global.get $FALSE)
          (local.get $offset))))))
