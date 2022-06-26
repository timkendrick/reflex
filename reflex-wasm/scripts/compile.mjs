// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';

import { print } from './transform/printer.mjs';
import { createLoaderContext, loadModule } from './transform/loader.mjs';
import transformDirectives from './transform/transforms/directives.mjs';
import { ParseError } from './transform/utils.mjs';
import branchDirective, { BRANCH_DIRECTIVE } from './transform/directives/branch.mjs';
import charDirective, { CHAR_DIRECTIVE } from './transform/directives/char.mjs';
import concatDirective, { CONCAT_DIRECTIVE } from './transform/directives/concat.mjs';
import foldDirective, { FOLD_DIRECTIVE } from './transform/directives/fold.mjs';
import getDirective, { GET_DIRECTIVE } from './transform/directives/get.mjs';
import lengthDirective, { LENGTH_DIRECTIVE } from './transform/directives/length.mjs';
import letDirective, { LET_DIRECTIVE } from './transform/directives/let.mjs';
import mapDirective, { MAP_DIRECTIVE } from './transform/directives/map.mjs';
import methodDirective, { METHOD_DIRECTIVE } from './transform/directives/method.mjs';
import includeDirective, { INCLUDE_DIRECTIVE } from './transform/directives/include.mjs';
import listDirective, { LIST_DIRECTIVE } from './transform/directives/list.mjs';
import switchDirective, { SWITCH_DIRECTIVE } from './transform/directives/switch.mjs';
import storeBytesDirective, { STORE_BYTES_DIRECTIVE } from './transform/directives/store_bytes.mjs';
import zipDirective, { ZIP_DIRECTIVE } from './transform/directives/zip.mjs';

const args = process.argv.slice(2);
try {
  if (args.length === 0) throw new Error('Missing entry point argument');
  if (args.length > 1) throw new Error('Multiple entry point arguments');
  const [entryPoint] = args;
  const modulePath = path.join(process.cwd(), entryPoint);
  const context = createLoaderContext(modulePath, {
    transform: composeTransforms(
      transformDirectives({
        [BRANCH_DIRECTIVE]: branchDirective,
        [CHAR_DIRECTIVE]: charDirective,
        [CONCAT_DIRECTIVE]: concatDirective,
        [FOLD_DIRECTIVE]: foldDirective,
        [GET_DIRECTIVE]: getDirective,
        [INCLUDE_DIRECTIVE]: includeDirective,
        [LET_DIRECTIVE]: letDirective,
        [LENGTH_DIRECTIVE]: lengthDirective,
        [LIST_DIRECTIVE]: listDirective,
        [MAP_DIRECTIVE]: mapDirective,
        [METHOD_DIRECTIVE]: methodDirective,
        [SWITCH_DIRECTIVE]: switchDirective,
        [STORE_BYTES_DIRECTIVE]: storeBytesDirective,
        [ZIP_DIRECTIVE]: zipDirective,
      }),
    ),
  });
  const ast = loadModule(modulePath, context, {});
  const output = print(ast, context.sources);
  process.stdout.write(output);
  process.exit(0);
} catch (err) {
  if (err instanceof ParseError) {
    process.stderr.write(`${err.message}\n`);
  } else {
    console.error(err);
  }
  process.exit(1);
}

function composeTransforms(...transforms) {
  return (ast, context) => transforms.reduce((ast, transform) => transform(ast, context), ast);
}
