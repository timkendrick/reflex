// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import empty from './empty.test.mjs';
import evaluate from './evaluate.test.mjs';
import filter from './filter.test.mjs';
import flatten from './flatten.test.mjs';
import hashmapKeys from './hashmap_keys.test.mjs';
import hashmapValues from './hashmap_values.test.mjs';
import integers from './integers.test.mjs';
import indexedAccessor from './indexed_accessor.test.mjs';
import intersperse from './intersperse.test.mjs';
import map from './map.test.mjs';
import once from './once.test.mjs';
import range from './range.test.mjs';
import repeat from './repeat.test.mjs';
import skip from './skip.test.mjs';
import take from './take.test.mjs';
import zip from './zip.test.mjs';

export default (describe) => {
  empty(describe);
  evaluate(describe);
  filter(describe);
  flatten(describe);
  hashmapKeys(describe);
  hashmapValues(describe);
  indexedAccessor(describe);
  integers(describe);
  intersperse(describe);
  map(describe);
  once(describe);
  range(describe);
  repeat(describe);
  skip(describe);
  take(describe);
  zip(describe);
};
