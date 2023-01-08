;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(@map $global
  (@get $globals)
  (@block
    (call (@list_item (@get $global) 2))
    (global.set (@list_item (@get $global) 0))))
