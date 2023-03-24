// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import createWasmTestRunner from './node.runner.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const module_path = process.env.WASM_MODULE || path.join(__dirname, `../build/runtime.wasm`);
const ENTRY_POINT = getRequiredEnvVar('ENTRY_POINT');
const STATE = getOptionalPointerEnvVar('STATE');
const runner = createWasmTestRunner(module_path);

runner((describe) => {
  describe('<debug>', (test) => {
    test('Evaluate expression', (assert, runtime) => {
      debugger;
      let state = STATE === null ? runtime.NULL : STATE;
      const [result, dependencies] = runtime.exports[ENTRY_POINT](state);
      console.log(runtime.format(result));
      debugger;
    });
  });
}).then((success) => {
  process.exit(success ? 0 : 1);
});

function parsePointer(input) {
  const value = Number(input);
  if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) {
    throw new Error(`Invalid pointer address: ${input}`);
  }
  return value;
}

function getRequiredEnvVar(key) {
  const value = getOptionalEnvVar(key);
  if (value === null) {
    throw new Error(`Missing ${key} environment variable`);
  }
  return value;
}

function getOptionalEnvVar(key) {
  const value = process.env[key];
  if (value === undefined) return null;
  return value;
}

function getOptionalPointerEnvVar(key) {
  const value = getOptionalEnvVar(key);
  if (value === null) return null;
  return parsePointer(value);
}
