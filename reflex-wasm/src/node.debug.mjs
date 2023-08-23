// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import createWasmTestRunner from './node.runner.mjs';

const WASM_MODULE = getRequiredEnvVar('WASM_MODULE');
const ENTRY_POINT = getRequiredEnvVar('ENTRY_POINT');
const STATE = getOptionalPointerEnvVar('STATE');
const runner = createWasmTestRunner(WASM_MODULE);

runner((describe) => {
  describe('<debug>', (test) => {
    test('Evaluate expression', (assert, runtime) => {
      const DEBUG_LABEL = 'Evaluate';
      debugger;
      console.time(DEBUG_LABEL);
      console.profile(DEBUG_LABEL);
      const run = runtime.exports[ENTRY_POINT];
      const [result, dependencies] = STATE !== null ? run(STATE) : run();
      console.profileEnd(DEBUG_LABEL);
      console.timeEnd(DEBUG_LABEL);
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

function getRequiredPointerEnvVar(key) {
  const value = getRequiredEnvVar(key);
  return parsePointer(value);
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
