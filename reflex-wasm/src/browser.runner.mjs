// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { createTestRunner } from './runner.mjs';
import { createRuntime } from './runtime.mjs';
import WASI from './wasi.polyfill.mjs';

import imports from './imports.mjs';

export default (modulePath) => {
  const module = fetch(modulePath)
    .then((response) => response.arrayBuffer())
    .then((buffer) => WebAssembly.compile(buffer));
  return createTestRunner(
    async () => {
      const wasi = new WASI();
      const instance = await WebAssembly.instantiate(await module, imports(wasi));
      wasi.initialize(instance);
      return createRuntime(instance.exports);
    },
    {
      assert: {
        ok(value) {
          if (!value) throw new Error(`Assertion failed: ${value}`);
        },
        strictEqual(left, right) {
          if (left !== right) throw new Error(`Assertion failed: ${left} === ${right}`);
        },
        notStrictEqual(left, right) {
          if (left === right) throw new Error(`Assertion failed: ${left} !== ${right}`);
        },
      },
      hrtime: {
        bigint:
          typeof performance === 'object' && typeof performance.now === 'function'
            ? () => BigInt(Math.round(performance.now() * 1000 * 1000))
            : () => BigInt(Date.now() * 1000 * 1000),
      },
      print: (output) => console.log(output),
    },
  );
};
