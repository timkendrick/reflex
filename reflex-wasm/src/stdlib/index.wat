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
  (@include "./identity.wat")
  (@include "./if.wat")
  (@include "./if_error.wat")
  (@include "./if_pending.wat")
  (@include "./iterate.wat")
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
  (@include "./resolve_deep.wat")
  (@include "./resolve_shallow.wat")
  (@include "./round.wat")
  (@include "./sequence.wat")
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
      $Stdlib_Divide
      $Stdlib_Effect
      $Stdlib_EndsWith
      $Stdlib_Eq
      $Stdlib_Equal
      $Stdlib_Floor
      $Stdlib_Fold
      $Stdlib_Get
      $Stdlib_ResolveQueryBranch
      $Stdlib_ResolveQueryLeaf
      $Stdlib_Gt
      $Stdlib_Gte
      $Stdlib_Has
      $Stdlib_Hash
      $Stdlib_Identity
      $Stdlib_If
      $Stdlib_IfError
      $Stdlib_IfPending
      $Stdlib_Iterate
      $Stdlib_ParseJson
      $Stdlib_StringifyJson
      $Stdlib_Keys
      $Stdlib_Length
      $Stdlib_Lt
      $Stdlib_Lte
      $Stdlib_Max
      $Stdlib_Min
      $Stdlib_Multiply
      $Stdlib_Not
      $Stdlib_Or
      $Stdlib_Pow
      $Stdlib_Push
      $Stdlib_PushFront
      $Stdlib_Remainder
      $Stdlib_Replace
      $Stdlib_ResolveDeep
      $Stdlib_ResolveShallow
      $Stdlib_Round
      $Stdlib_Set
      $Stdlib_Sequence
      $Stdlib_Skip
      $Stdlib_Slice
      $Stdlib_Split
      $Stdlib_StartsWith
      $Stdlib_Subtract
      $Stdlib_Take
      $Stdlib_Values
      $Stdlib_Zip)

    (func $Builtin::apply (param $target i32) (param $args i32) (param $state i32) (result i32 i32)
      (call_indirect (type $Builtin)
        (local.get $args)
        (local.get $state)
        (local.get $target)))

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
