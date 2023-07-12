// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

import pkg from '../package.json';

export { default as configs } from './configs';
export { default as rules } from './rules';

export const meta = {
  name: pkg.name,
  version: pkg.version,
};
