;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (memory 1)
  (@let $Nil
    (@struct $Nil)

    (@block
      (@constructor (@get $Nil))
      (@derive $size (@get $Nil))
      (@derive $equals (@get $Nil))
      (@derive $hash (@get $Nil))))

  (@let $Int
    (@struct $Int
      (@field $value i32))

    (@block
      (@constructor (@get $Int))
      (@derive $size (@get $Int))
      (@derive $equals (@get $Int))
      (@derive $hash (@get $Int))))

  (@let $Tree
    (@struct $Tree
      (@field $left (@ref $Term @optional))
      (@field $right (@ref $Term @optional))
      (@field $length i32))

    (@block
      (@constructor (@get $Tree))
      (@derive $size (@get $Tree))
      (@derive $equals (@get $Tree))
      (@derive $hash (@get $Tree))))

  (@let $List
    (@struct $List
      (@field $items (@repeated (@ref $Term))))

    (@block
      (@constructor (@get $List))
      (@derive $size (@get $List))
      (@derive $equals (@get $List))
      (@derive $hash (@get $List))))

  (@let $Hashmap
    (@struct $Hashmap
      (@field $num_entries i32)
      (@field $buckets
        (@repeated
          (@struct $HashmapBucket
            (@field $key (@ref $Term))
            (@field $value (@ref $Term))))))

    (@block
      (@constructor (@get $Hashmap))
      (@derive $size (@get $Hashmap))
      (@derive $equals (@get $Hashmap))
      (@derive $hash (@get $Hashmap))))

  (@let $Condition
    (@union $Condition

      (@struct $CustomCondition
        (@field $effect_type (@ref $Term))
        (@field $effect_payload (@ref $Term)))

      (@struct $PendingCondition)

      (@struct $ErrorCondition
        (@field $effect_payload (@ref $Term)))

      (@struct $SizedCondition
        (@field $value f64)
        (@field $effect_payload (@repeated (@struct $Term (@field $foo i32) (@field $bar f64))))) )

    (@block
      (@constructor (@get $Condition))
      (@derive $size (@get $Condition))
      (@derive $equals (@get $Condition))
      (@derive $hash (@get $Condition)))))
