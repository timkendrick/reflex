// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { CONST_INITIALIZERS_GLOBAL } from './const.mjs';
import { createListDirective } from './list.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/const_init.wat');

export const CONST_INIT_DIRECTIVE = '@const-init';

export default function constInitDirective(node, context) {
  const [instruction, ...varArgs] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .filter((node) => !isNonFunctionalNode(node));
  if (!isNamedTermNode(CONST_INIT_DIRECTIVE, instruction) || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONST_INIT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const initializers = sortInitializers(
    context.globals.get(CONST_INITIALIZERS_GLOBAL) || new Map(),
  );
  return getTemplateElements(
    context.import(TEMPLATE, {
      $initializers: createListDirective({
        elements: initializers,
        location: node.location,
      }),
    }).module,
  );
}

function sortInitializers(initializers) {
  const entries = Array.from(initializers.entries());
  const queue = entries.slice();
  const depths = new Map();
  while (queue.length > 0) {
    const entry = queue.shift();
    const [name, { dependencies }] = entry;
    const depth = dependencies.reduce((depth, name) => {
      const dependencyDepth = depth === null ? undefined : depths.get(name);
      if (dependencyDepth === undefined) return null;
      return Math.max(depth, dependencyDepth + 1);
    }, 0);
    if (depth === null) {
      queue.push(entry);
      break;
    }
    depths.set(name, depth);
  }
  return entries
    .sort((a, b) => depths.get(a[0]) - depths.get(b[0]))
    .map(([_, { identifier }]) => identifier);
}

function getTemplateElements(template) {
  return template.statements;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
