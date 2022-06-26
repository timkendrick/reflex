// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import createWasmTestRunner from './browser.runner.mjs';

import tests from './index.test.mjs';

const runner = createWasmTestRunner('../build/runtime.debug.wasm');

runner(tests).then((success) => {
  document.body.appendChild(document.createTextNode(success ? 'Tests passed' : 'Tests failed'));
});
