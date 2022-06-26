// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import hashmap from './hashmap.test.mjs';
import list from './list.test.mjs';

export default (describe) => {
  hashmap(describe);
  list(describe);
};
