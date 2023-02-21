// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';

import { print } from './transform/printer.mjs';
import { createLoaderContext, loadModule } from './transform/loader.mjs';
import transformDirectives from './transform/transforms/directives.mjs';
import { ParseError } from './transform/utils.mjs';
import addDirective, { ADD_DIRECTIVE } from './transform/directives/add.mjs';
import blockDirective, { BLOCK_DIRECTIVE } from './transform/directives/block.mjs';
import branchDirective, { BRANCH_DIRECTIVE } from './transform/directives/branch.mjs';
import builtinDirective, { BUILTIN_DIRECTIVE } from './transform/directives/builtin.mjs';
import chainDirective, { CHAIN_DIRECTIVE } from './transform/directives/chain.mjs';
import charDirective, { CHAR_DIRECTIVE } from './transform/directives/char.mjs';
import concatDirective, { CONCAT_DIRECTIVE } from './transform/directives/concat.mjs';
import constDirective, { CONST_DIRECTIVE } from './transform/directives/const.mjs';
import constInitDirective, { CONST_INIT_DIRECTIVE } from './transform/directives/const_init.mjs';
import constStringDirective, { CONST_STRING_DIRECTIVE } from './transform/directives/const_string.mjs';
import constructorDirective, { CONSTRUCTOR_DIRECTIVE } from './transform/directives/constructor.mjs';
import delegateDirective, { DELEGATE_DIRECTIVE } from './transform/directives/delegate.mjs';
import deriveDirective, { DERIVE_DIRECTIVE } from './transform/directives/derive.mjs';
import exportDirective, { EXPORT_DIRECTIVE } from './transform/directives/export.mjs';
import fieldDirective, { FIELD_DIRECTIVE } from './transform/directives/field.mjs';
import foldDirective, { FOLD_DIRECTIVE } from './transform/directives/fold.mjs';
import getDirective, { GET_DIRECTIVE } from './transform/directives/get.mjs';
import importDirective, { IMPORT_DIRECTIVE } from './transform/directives/import.mjs';
import includeDirective, { INCLUDE_DIRECTIVE } from './transform/directives/include.mjs';
import instructionDirective, { INSTRUCTION_DIRECTIVE } from './transform/directives/instruction.mjs';
import iterateDirective, { ITERATE_DIRECTIVE } from './transform/directives/iterate.mjs';
import iterateMapDirective, { ITERATE_MAP_DIRECTIVE } from './transform/directives/iterate_map.mjs';
import lengthDirective, { LENGTH_DIRECTIVE } from './transform/directives/length.mjs';
import letDirective, { LET_DIRECTIVE } from './transform/directives/let.mjs';
import listDirective, { LIST_DIRECTIVE } from './transform/directives/list.mjs';
import listItemDirective, { LIST_ITEM_DIRECTIVE } from './transform/directives/list_item.mjs';
import mapDirective, { MAP_DIRECTIVE } from './transform/directives/map.mjs';
import reverseDirective, { REVERSE_DIRECTIVE } from './transform/directives/reverse.mjs';
import signaturesDirective, { SIGNATURES_DIRECTIVE } from './transform/directives/signatures.mjs';
import storeBytesDirective, { STORE_BYTES_DIRECTIVE } from './transform/directives/store_bytes.mjs';
import structDirective, { STRUCT_DIRECTIVE } from './transform/directives/struct.mjs';
import stringLengthDirective, { STRING_LENGTH_DIRECTIVE } from './transform/directives/string_length.mjs';
import switchDirective, { SWITCH_DIRECTIVE } from './transform/directives/switch.mjs';
import toStringDirective, { TO_STRING_DIRECTIVE } from './transform/directives/to_string.mjs';
import unionDirective, { UNION_DIRECTIVE } from './transform/directives/union.mjs';
import unionVariantDirective, { UNION_VARIANT_DIRECTIVE } from './transform/directives/union_variant.mjs';
import unionVariantsDirective, { UNION_VARIANTS_DIRECTIVE } from './transform/directives/union_variants.mjs';
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
        [ADD_DIRECTIVE]: addDirective,
        [BLOCK_DIRECTIVE]: blockDirective,
        [BRANCH_DIRECTIVE]: branchDirective,
        [BUILTIN_DIRECTIVE]: builtinDirective,
        [CHAIN_DIRECTIVE]: chainDirective,
        [CHAR_DIRECTIVE]: charDirective,
        [CONCAT_DIRECTIVE]: concatDirective,
        [CONST_DIRECTIVE]: constDirective,
        [CONST_INIT_DIRECTIVE]: constInitDirective,
        [CONST_STRING_DIRECTIVE]: constStringDirective,
        [CONSTRUCTOR_DIRECTIVE]: constructorDirective,
        [DELEGATE_DIRECTIVE]: delegateDirective,
        [DERIVE_DIRECTIVE]: deriveDirective,
        [FIELD_DIRECTIVE]: fieldDirective,
        [FOLD_DIRECTIVE]: foldDirective,
        [GET_DIRECTIVE]: getDirective,
        [IMPORT_DIRECTIVE]: importDirective,
        [EXPORT_DIRECTIVE]: exportDirective,
        [INCLUDE_DIRECTIVE]: includeDirective,
        [INSTRUCTION_DIRECTIVE]: instructionDirective,
        [ITERATE_DIRECTIVE]: iterateDirective,
        [ITERATE_MAP_DIRECTIVE]: iterateMapDirective,
        [LENGTH_DIRECTIVE]: lengthDirective,
        [LET_DIRECTIVE]: letDirective,
        [LIST_DIRECTIVE]: listDirective,
        [LIST_ITEM_DIRECTIVE]: listItemDirective,
        [MAP_DIRECTIVE]: mapDirective,
        [REVERSE_DIRECTIVE]: reverseDirective,
        [SIGNATURES_DIRECTIVE]: signaturesDirective,
        [STORE_BYTES_DIRECTIVE]: storeBytesDirective,
        [STRING_LENGTH_DIRECTIVE]: stringLengthDirective,
        [STRUCT_DIRECTIVE]: structDirective,
        [SWITCH_DIRECTIVE]: switchDirective,
        [TO_STRING_DIRECTIVE]: toStringDirective,
        [UNION_DIRECTIVE]: unionDirective,
        [UNION_VARIANT_DIRECTIVE]: unionVariantDirective,
        [UNION_VARIANTS_DIRECTIVE]: unionVariantsDirective,
        [ZIP_DIRECTIVE]: zipDirective,
      }),
    ),
  });
  const ast = loadModule(modulePath, context, {}).module;
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
