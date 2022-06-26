// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import application from './application.test.mjs';
import boolean from './boolean.test.mjs';
import builtin from './builtin.test.mjs';
import cell from './cell.test.mjs';
import collection from './collection/index.test.mjs';
import condition from './condition.test.mjs';
import effect from './effect.test.mjs';
import float from './float.test.mjs';
import int from './int.test.mjs';
import iterator from './iterator/index.test.mjs';
import lambda from './lambda.test.mjs';
import _let from './let.test.mjs';
import nil from './nil.test.mjs';
import partial from './partial.test.mjs';
import pointer from './pointer.test.mjs';
import signal from './signal.test.mjs';
import string from './string.test.mjs';
import symbol from './symbol.test.mjs';
import variable from './variable.test.mjs';

export default (describe) => {
  application(describe);
  boolean(describe);
  builtin(describe);
  cell(describe);
  collection(describe);
  condition(describe);
  effect(describe);
  float(describe);
  int(describe);
  lambda(describe);
  _let(describe);
  nil(describe);
  partial(describe);
  pointer(describe);
  signal(describe);
  string(describe);
  symbol(describe);
  variable(describe);
  iterator(describe);
};
