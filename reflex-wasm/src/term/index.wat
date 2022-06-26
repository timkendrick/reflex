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
  (@include "./lambda.wat")
  (@include "./let.wat")
  (@include "./nil.wat")
  (@include "./partial.wat")
  (@include "./pointer.wat")
  (@include "./signal.wat")
  (@include "./string.wat")
  (@include "./symbol.wat")
  (@include "./variable.wat")

  (@let $TermType
    (@union $TermType
      (@import $Application "./application.wat")
      (@import $Boolean "./boolean.wat")
      (@import $Builtin "./builtin.wat")
      (@import $Cell "./cell.wat")
      (@import $Hashmap "./collection/hashmap.wat")
      (@import $Lambda "./lambda.wat")
      (@import $List "./collection/list.wat")
      (@import $Record "./collection/record.wat")
      (@import $Tree "./collection/tree.wat")
      (@import $Condition "./condition.wat")
      (@import $Effect "./effect.wat")
      (@import $Float "./float.wat")
      (@import $Int "./int.wat")
      (@import $Nil "./nil.wat")
      (@import $Partial "./partial.wat")
      (@import $Pointer "./pointer.wat")
      (@import $Signal "./signal.wat")
      (@import $String "./string.wat")
      (@import $Symbol "./symbol.wat")
      (@import $Variable "./variable.wat")
      (@import $Let "./let.wat")
      (@import $EmptyIterator "./iterator/empty.wat")
      (@import $EvaluateIterator "./iterator/evaluate.wat")
      (@import $FilterIterator "./iterator/filter.wat")
      (@import $FlattenIterator "./iterator/flatten.wat")
      (@import $HashmapKeysIterator "./iterator/hashmap_keys.wat")
      (@import $HashmapValuesIterator "./iterator/hashmap_values.wat")
      (@import $IntegersIterator "./iterator/integers.wat")
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

    ;; Trait implementations
    ;; TODO: Refactor manual trait delegation implementations into macro
    (@let $trait_typenames
      (@list
        $Application
        $Effect
        $Let
        $Pointer)

      (func $Term::implements::evaluate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

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
        $Lambda
        $Signal)

      (func $Term::implements::apply (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

      (func $Term::traits::apply (param $self i32) (param $args i32) (param $state i32) (result i32 i32)
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
        $Tree
        $EmptyIterator
        $OnceIterator
        $RepeatIterator
        $SkipIterator
        $TakeIterator
        $ZipIterator
        $MapIterator
        $FilterIterator
        $FlattenIterator
        $EvaluateIterator
        $IntegersIterator
        $RangeIterator
        $HashmapKeysIterator
        $HashmapValuesIterator)

      (func $Term::implements::iterate (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

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

      (func $Term::implements::size_hint (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

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

      (func $Term::implements::next (param $type i32) (result i32)
        (@fold $result $typename
          (@get $trait_typenames)
          (global.get $FALSE)
          (i32.or
            (@get $result)
            (i32.eq (local.get $type) (global.get (@concat "$TermType::" (@get $typename)))))))

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
          (global.get $NULL))))))
