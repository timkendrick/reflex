// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import application from './application.test.mjs';
import boolean from './boolean.test.mjs';
import builtin from './builtin.test.mjs';
import cell from './cell.test.mjs';
import condition from './condition.test.mjs';
import constructor from './constructor.test.mjs';
import effect from './effect.test.mjs';
import float from './float.test.mjs';
import hashmap from './hashmap.test.mjs';
import hashset from './hashset.test.mjs';
import int from './int.test.mjs';
import iterator from './iterator/index.test.mjs';
import lambda from './lambda.test.mjs';
import lazyResult from './lazy_result.test.mjs';
import _let from './let.test.mjs';
import list from './list.test.mjs';
import nil from './nil.test.mjs';
import partial from './partial.test.mjs';
import pointer from './pointer.test.mjs';
import record from './record.test.mjs';
import signal from './signal.test.mjs';
import string from './string.test.mjs';
import symbol from './symbol.test.mjs';
import timestamp from './timestamp.test.mjs';
import tree from './tree.test.mjs';
import variable from './variable.test.mjs';

export default (describe) => {
  application(describe);
  boolean(describe);
  builtin(describe);
  cell(describe);
  condition(describe);
  constructor(describe);
  effect(describe);
  float(describe);
  hashmap(describe);
  hashset(describe);
  int(describe);
  lambda(describe);
  lazyResult(describe);
  _let(describe);
  list(describe);
  nil(describe);
  partial(describe);
  pointer(describe);
  record(describe);
  signal(describe);
  string(describe);
  symbol(describe);
  timestamp(describe);
  tree(describe);
  variable(describe);
  iterator(describe);
};
