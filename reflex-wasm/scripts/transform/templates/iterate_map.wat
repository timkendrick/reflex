;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
;; Determine whether the source iterator size is known
(if (result i32 i32)
  (i32.eq (local.tee (@get $length) (call $Term::traits::size_hint (local.get (@get $source)))) (global.get $NULL))
  (then
    ;; If the source iterator size is unknown, allocate a new dynamic list
    (local.tee (@get $result) (call $Term::List::allocate_unsized))
    ;; Iterate through the source iterator items
    (local.set (@get $index) (i32.const 0))
    (@iterate (@get $source) (@get $item) (@get $iterator_state) (@get $state) (@get $dependencies)
      ;; Prepare the stack for pushing items onto the output list
      (local.get (@get $result))
      ;; Inject the iteratee body
      (@get $body)
      ;; Update the accumuated dependencies
      (local.set (@get $dependencies) (call $Dependencies::traits::union (local.get (@get $dependencies))))
      ;; Push the transformed item onto the output list currently present at the top of the stack
      (call $Term::List::append_unsized)
      ;; Update the iteration index
      (local.set (@get $index) (i32.add (local.get (@get $index)) (i32.const 1))))
    ;; Initialize the dynamic list term
    (call $Term::List::init_unsized)
    (local.get (@get $dependencies)))
  (else
    ;; Otherwise if the source iterator size is known, allocate a new list of the correct length
    (local.tee (@get $result) (call $Term::List::allocate (local.get (@get $length))))
    ;; Iterate through the source iterator items
    (local.set (@get $index) (i32.const 0))
    (@iterate (@get $source) (@get $item) (@get $iterator_state) (@get $state) (@get $dependencies)
      ;; Prepare the stack for pushing items onto the output list
      (local.get (@get $result))
      (local.get (@get $index))
      ;; Inject the iteratee body
      (@get $body)
      ;; Update the accumuated dependencies
      (local.set (@get $dependencies) (call $Dependencies::traits::union (local.get (@get $dependencies))))
      ;; Push the transformed item onto the output list currently present at the top of the stack
      (call $Term::List::set_item)
      ;; Update the iteration index
      (local.set (@get $index) (i32.add (local.get (@get $index)) (i32.const 1))))
    ;; Initialize the list term
    (call $Term::List::init (local.get (@get $length)))
    (local.get (@get $dependencies))))
