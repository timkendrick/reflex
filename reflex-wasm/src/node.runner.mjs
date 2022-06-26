// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import assert from 'assert';
import fs from 'fs';
import { argv, env, hrtime } from 'process';
import { WASI } from 'wasi';

import imports from './imports.mjs';
import { createTestRunner } from './runner.mjs';
import { createRuntime } from './runtime.mjs';

export default (modulePath) => {
  const buffer = fs.readFileSync(modulePath);
  const module = WebAssembly.compile(buffer);
  return createTestRunner(
    async () => {
      const wasi = new WASI({
        args: argv,
        env,
      });
      const instance = await WebAssembly.instantiate(
        await module,
        imports(wasi, () => instance),
      );
      wasi.initialize(instance);
      return createRuntime(instance.exports);
    },
    {
      assert,
      hrtime,
      print: (output) => process.stdout.write(output),
    },
  );
};
