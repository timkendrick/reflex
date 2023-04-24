// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import createWasmTestRunner from './browser.runner.mjs';

const module_path = getOptionalEnvVar('WASM_MODULE') || `../build/runtime.wasm`;
const ENTRY_POINT = getRequiredPointerEnvVar('ENTRY_POINT');
const STATE = getOptionalPointerEnvVar('STATE');
const runner = createWasmTestRunner(module_path);

runner((describe) => {
  describe('<debug>', (test) => {
    test('Evaluate expression', (assert, runtime) => {
      const DEBUG_LABEL = 'Evaluate';
      const state = STATE === null ? runtime.NULL : STATE;
      debugger;
      console.time(DEBUG_LABEL);
      console.profile(DEBUG_LABEL);
      const [result, dependencies] = runtime.exports.evaluate(ENTRY_POINT, state);
      console.profileEnd(DEBUG_LABEL);
      console.timeEnd(DEBUG_LABEL);
      console.log(runtime.format(result));
      debugger;
    });
  });
}).then((success) => {
  document.body.appendChild(document.createTextNode(success ? 'Tests passed' : 'Tests failed'));
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
  const value = new URLSearchParams(document.location.search).get(key);
  if (value === undefined) return null;
  return value;
}

function getOptionalPointerEnvVar(key) {
  const value = getOptionalEnvVar(key);
  if (value === null) return null;
  return parsePointer(value);
}
