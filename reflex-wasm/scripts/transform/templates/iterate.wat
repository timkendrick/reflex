;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
;; Initialize the iterator state
(local.set (@get $iterator_state) (global.get $NULL))
;; Iterate through each of the source iterator items
(loop $LOOP
  ;; Consume the next iterator item
  (call $Term::traits::next (local.get (@get $source)) (local.get (@get $iterator_state)) (local.get (@get $state)))
  ;; Update the accumulated dependencies
  (local.set (@get $dependencies) (call $Dependencies::traits::union (local.get (@get $dependencies))))
  ;; Update the iterator state
  (local.set (@get $iterator_state))
  (if
    ;; If this was the final item, nothing more to do
    (i32.eq (local.tee (@get $item)) (global.get $NULL))
    (then)
    (else
      ;; Otherwise evaluate the provided iteratee body
      (@get $body)
      ;; Continue with the next item
      (br $LOOP))))
