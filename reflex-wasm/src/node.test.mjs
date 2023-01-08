// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import createWasmTestRunner from './node.runner.mjs';

import tests from './index.test.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const runner = createWasmTestRunner(path.join(__dirname, `../build/runtime.wasm`));

runner(tests).then((success) => {
  process.exit(success ? 0 : 1);
});
