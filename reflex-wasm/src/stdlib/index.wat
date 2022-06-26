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

  (global $Stdlib_Abs (export "Stdlib_Abs") i32 (i32.const 0))
  (global $Stdlib_Add (export "Stdlib_Add") i32 (i32.const 1))
  (global $Stdlib_And (export "Stdlib_And") i32 (i32.const 2))
  (global $Stdlib_Apply (export "Stdlib_Apply") i32 (i32.const 3))
  (global $Stdlib_Car (export "Stdlib_Car") i32 (i32.const 4))
  (global $Stdlib_Cdr (export "Stdlib_Cdr") i32 (i32.const 5))
  (global $Stdlib_Ceil (export "Stdlib_Ceil") i32 (i32.const 6))
  (global $Stdlib_Chain (export "Stdlib_Chain") i32 (i32.const 7))
  (global $Stdlib_CollectHashmap (export "Stdlib_CollectHashmap") i32 (i32.const 8))
  (global $Stdlib_CollectList (export "Stdlib_CollectList") i32 (i32.const 9))
  (global $Stdlib_CollectString (export "Stdlib_CollectString") i32 (i32.const 10))
  (global $Stdlib_CollectTree (export "Stdlib_CollectTree") i32 (i32.const 11))
  (global $Stdlib_Cons (export "Stdlib_Cons") i32 (i32.const 12))
  (global $Stdlib_Divide (export "Stdlib_Divide") i32 (i32.const 13))
  (global $Stdlib_Effect (export "Stdlib_Effect") i32 (i32.const 14))
  (global $Stdlib_EndsWith (export "Stdlib_EndsWith") i32 (i32.const 15))
  (global $Stdlib_Eq (export "Stdlib_Eq") i32 (i32.const 16))
  (global $Stdlib_Equal (export "Stdlib_Equal") i32 (i32.const 17))
  (global $Stdlib_Floor (export "Stdlib_Floor") i32 (i32.const 18))
  (global $Stdlib_Fold (export "Stdlib_Fold") i32 (i32.const 19))
  (global $Stdlib_Get (export "Stdlib_Get") i32 (i32.const 20))
  (global $Stdlib_Gt (export "Stdlib_Gt") i32 (i32.const 21))
  (global $Stdlib_Gte (export "Stdlib_Gte") i32 (i32.const 22))
  (global $Stdlib_Has (export "Stdlib_Has") i32 (i32.const 23))
  (global $Stdlib_Hash (export "Stdlib_Hash") i32 (i32.const 24))
  (global $Stdlib_Identity (export "Stdlib_Identity") i32 (i32.const 25))
  (global $Stdlib_If (export "Stdlib_If") i32 (i32.const 26))
  (global $Stdlib_IfError (export "Stdlib_IfError") i32 (i32.const 27))
  (global $Stdlib_IfPending (export "Stdlib_IfPending") i32 (i32.const 28))
  (global $Stdlib_Iterate (export "Stdlib_Iterate") i32 (i32.const 29))
  (global $Stdlib_Keys (export "Stdlib_Keys") i32 (i32.const 30))
  (global $Stdlib_Length (export "Stdlib_Length") i32 (i32.const 31))
  (global $Stdlib_Lt (export "Stdlib_Lt") i32 (i32.const 32))
  (global $Stdlib_Lte (export "Stdlib_Lte") i32 (i32.const 33))
  (global $Stdlib_Max (export "Stdlib_Max") i32 (i32.const 34))
  (global $Stdlib_Min (export "Stdlib_Min") i32 (i32.const 35))
  (global $Stdlib_Multiply (export "Stdlib_Multiply") i32 (i32.const 36))
  (global $Stdlib_Not (export "Stdlib_Not") i32 (i32.const 37))
  (global $Stdlib_Or (export "Stdlib_Or") i32 (i32.const 38))
  (global $Stdlib_ParseJson (export "Stdlib_ParseJson") i32 (i32.const 39))
  (global $Stdlib_Pow (export "Stdlib_Pow") i32 (i32.const 40))
  (global $Stdlib_Push (export "Stdlib_Push") i32 (i32.const 41))
  (global $Stdlib_PushFront (export "Stdlib_PushFront") i32 (i32.const 42))
  (global $Stdlib_Remainder (export "Stdlib_Remainder") i32 (i32.const 43))
  (global $Stdlib_Replace (export "Stdlib_Replace") i32 (i32.const 44))
  (global $Stdlib_ResolveDeep (export "Stdlib_ResolveDeep") i32 (i32.const 45))
  (global $Stdlib_ResolveShallow (export "Stdlib_ResolveShallow") i32 (i32.const 46))
  (global $Stdlib_Round (export "Stdlib_Round") i32 (i32.const 47))
  (global $Stdlib_Set (export "Stdlib_Set") i32 (i32.const 48))
  (global $Stdlib_Sequence (export "Stdlib_Sequence") i32 (i32.const 49))
  (global $Stdlib_Skip (export "Stdlib_Skip") i32 (i32.const 50))
  (global $Stdlib_Slice (export "Stdlib_Slice") i32 (i32.const 51))
  (global $Stdlib_Split (export "Stdlib_Split") i32 (i32.const 52))
  (global $Stdlib_StartsWith (export "Stdlib_StartsWith") i32 (i32.const 53))
  (global $Stdlib_Subtract (export "Stdlib_Subtract") i32 (i32.const 54))
  (global $Stdlib_Take (export "Stdlib_Take") i32 (i32.const 55))
  (global $Stdlib_Values (export "Stdlib_Values") i32 (i32.const 56))
  (global $Stdlib_Zip (export "Stdlib_Zip") i32 (i32.const 57))

  ;; Declare builtin function implementations
  (table (export "__indirect_function_table") 59 funcref)
  (elem (i32.const 0)
    $Stdlib_Abs
    $Stdlib_Add
    $Stdlib_And
    $Stdlib_Apply
    $Stdlib_Car
    $Stdlib_Cdr
    $Stdlib_Ceil
    $Stdlib_Chain
    $Stdlib_CollectHashmap
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
    $Stdlib_Gt
    $Stdlib_Gte
    $Stdlib_Has
    $Stdlib_Hash
    $Stdlib_Identity
    $Stdlib_If
    $Stdlib_IfError
    $Stdlib_IfPending
    $Stdlib_Iterate
    $Stdlib_Keys
    $Stdlib_Length
    $Stdlib_Lt
    $Stdlib_Lte
    $Stdlib_Max
    $Stdlib_Min
    $Stdlib_Multiply
    $Stdlib_Not
    $Stdlib_Or
    $Stdlib_ParseJson
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
    $Stdlib_Zip))
