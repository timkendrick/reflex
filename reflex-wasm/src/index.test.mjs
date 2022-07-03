// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import stdlib from './stdlib/index.test.mjs';
import termType from './term_type/index.test.mjs';

export default (describe) => {
  termType(describe);
  stdlib(describe);
};
