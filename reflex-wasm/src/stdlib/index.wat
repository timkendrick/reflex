;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@include "./abs.wat")
  (@include "./add.wat")
  (@include "./and.wat")
  (@include "./apply.wat")
  (@include "./car.wat")
  (@include "./cdr.wat")
  (@include "./ceil.wat")
  (@include "./chain.wat")
  (@include "./collect_hashmap.wat")
  (@include "./collect_hashset.wat")
  (@include "./collect_list.wat")
  (@include "./collect_string.wat")
  (@include "./collect_tree.wat")
  (@include "./cons.wat")
  (@include "./divide.wat")
  (@include "./effect.wat")
  (@include "./ends_with.wat")
  (@include "./eq.wat")
  (@include "./equal.wat")
  (@include "./floor.wat")
  (@include "./fold.wat")
  (@include "./get.wat")
  (@include "./graphql/resolve_query_branch.wat")
  (@include "./graphql/resolve_query_leaf.wat")
  (@include "./gt.wat")
  (@include "./gte.wat")
  (@include "./has.wat")
  (@include "./hash.wat")
  (@include "./handlers/scan.wat")
  (@include "./handlers/to_request.wat")
  (@include "./handlers/variable.wat")
  (@include "./identity.wat")
  (@include "./if.wat")
  (@include "./if_error.wat")
  (@include "./if_pending.wat")
  (@include "./iterate.wat")
  (@include "./js/accessor.wat")
  (@include "./js/construct.wat")
  (@include "./js/debug.wat")
  (@include "./js/format_error_message.wat")
  (@include "./js/is_finite.wat")
  (@include "./js/log.wat")
  (@include "./js/parse_date.wat")
  (@include "./js/parse_float.wat")
  (@include "./js/parse_int.wat")
  (@include "./js/throw.wat")
  (@include "./js/to_string.wat")
  (@include "./js/urlencode.wat")
  (@include "./json/parse_json.wat")
  (@include "./json/stringify_json.wat")
  (@include "./keys.wat")
  (@include "./length.wat")
  (@include "./lt.wat")
  (@include "./lte.wat")
  (@include "./max.wat")
  (@include "./min.wat")
  (@include "./multiply.wat")
  (@include "./not.wat")
  (@include "./or.wat")
  (@include "./pow.wat")
  (@include "./push.wat")
  (@include "./push_front.wat")
  (@include "./remainder.wat")
  (@include "./replace.wat")
  (@include "./resolve_args.wat")
  (@include "./resolve_deep.wat")
  (@include "./resolve_shallow.wat")
  (@include "./round.wat")
  (@include "./sequence.wat")
  (@include "./server/graphql_resolver.wat")
  (@include "./set.wat")
  (@include "./skip.wat")
  (@include "./slice.wat")
  (@include "./split.wat")
  (@include "./starts_with.wat")
  (@include "./subtract.wat")
  (@include "./take.wat")
  (@include "./values.wat")
  (@include "./zip.wat")

  (@let $builtins
    (@list
      $Stdlib_Abs
      $Stdlib_Accessor
      $Stdlib_Add
      $Stdlib_And
      $Stdlib_Apply
      $Stdlib_Car
      $Stdlib_Cdr
      $Stdlib_Ceil
      $Stdlib_Chain
      $Stdlib_CollectHashmap
      $Stdlib_CollectHashset
      $Stdlib_CollectList
      $Stdlib_CollectString
      $Stdlib_CollectTree
      $Stdlib_Cons
      $Stdlib_Construct
      $Stdlib_Debug
      $Stdlib_DecrementVariable
      $Stdlib_Divide
      $Stdlib_Effect
      $Stdlib_EndsWith
      $Stdlib_Eq
      $Stdlib_Equal
      $Stdlib_Floor
      $Stdlib_Fold
      $Stdlib_FormatErrorMessage
      $Stdlib_Get
      $Stdlib_GetVariable
      $Stdlib_GraphQlResolver
      $Stdlib_Gt
      $Stdlib_Gte
      $Stdlib_Has
      $Stdlib_Hash
      $Stdlib_Identity
      $Stdlib_If
      $Stdlib_IfError
      $Stdlib_IfPending
      $Stdlib_IncrementVariable
      $Stdlib_IsFinite
      $Stdlib_Iterate
      $Stdlib_Keys
      $Stdlib_Length
      $Stdlib_Log
      $Stdlib_Lt
      $Stdlib_Lte
      $Stdlib_Max
      $Stdlib_Min
      $Stdlib_Multiply
      $Stdlib_Not
      $Stdlib_Or
      $Stdlib_ParseDate
      $Stdlib_ParseFloat
      $Stdlib_ParseInt
      $Stdlib_ParseJson
      $Stdlib_Pow
      $Stdlib_Push
      $Stdlib_PushFront
      $Stdlib_Remainder
      $Stdlib_Replace
      $Stdlib_ResolveArgs
      $Stdlib_ResolveDeep
      $Stdlib_ResolveQueryBranch
      $Stdlib_ResolveQueryLeaf
      $Stdlib_ResolveShallow
      $Stdlib_Round
      $Stdlib_Scan
      $Stdlib_Sequence
      $Stdlib_Set
      $Stdlib_SetVariable
      $Stdlib_Skip
      $Stdlib_Slice
      $Stdlib_Split
      $Stdlib_StartsWith
      $Stdlib_StringifyJson
      $Stdlib_Subtract
      $Stdlib_Take
      $Stdlib_Throw
      $Stdlib_ToRequest
      $Stdlib_ToString
      $Stdlib_Urlencode
      $Stdlib_Values
      $Stdlib_Zip)

    (func $Builtin::apply (param $target i32) (param $args i32) (param $state i32) (result i32 i32)
      (call_indirect (type $Builtin)
        (local.get $args)
        (local.get $state)
        (local.get $target)))

    (func $Builtin::arity (param $target i32) (result i32)
      (@branch
        (local.get $target)
        (@list
          (@map $builtin
            (@get $builtins)
            (return (call (@concat "$" (@get $builtin) "::arity")))))
        (i32.const 0)))

    (func $Builtin::display (param $target i32) (param $offset i32) (result i32)
      (@branch
        (local.get $target)
        (@list
          (@map $builtin
            (@get $builtins)
            (return (call (@concat "$" (@get $builtin) "::display") (local.get $offset)))))
        ;; Default implementation
        (@store-bytes $offset "<function:")
        (local.set $offset (i32.add (local.get $offset)))
        ;; Write the number of arguments to the output
        (call $Utils::u32::write_string
          (call $Builtin::arity (local.get $target))
          (local.get $offset))
        (local.set $offset (i32.add (local.get $offset)))
        (@store-bytes $offset ">")
        (i32.add (local.get $offset))))

    (@block
      ;; Declare builtin function implementations
      (@map $builtin
        (@get $builtins)
        (@block
          (global (@get $builtin) (export (@concat "\"" (@get $builtin) "\"")) i32 (i32.const (@get $_)))))

      (table (export "__indirect_function_table") (@length (@get $builtins)) funcref)
      (elem (i32.const 0)
        (@map $builtin
          (@get $builtins)
          (@block
            (@concat "$" (@get $builtin) "::apply")))))))
