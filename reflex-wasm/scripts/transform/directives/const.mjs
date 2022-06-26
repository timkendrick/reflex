// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createBlockDirective } from './block.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/const.wat');

export const CONST_INITIALIZERS_GLOBAL = '@const::initializers';

export const CONST_DIRECTIVE = '@const';
export const DEPENDS_ON_DIRECTIVE = '@depends-on';

export default function constDirective(node, context) {
  const [instruction, identifier, type, dependencies, ...initializer] = node.elements.reduce(
    (results, node) => {
      switch (results.length) {
        case 0:
        case 1:
        case 2:
          results.push(
            ...(context.transform ? context.transform(node, context) : [node]).filter(
              (node) => !isNonFunctionalNode(node),
            ),
          );
          return results;
        case 3:
          results.push([]);
        case 4:
          if (isNonFunctionalNode(node)) return results;
          if (isDependencyNode(node)) {
            results[results.length - 1].push(node);
            return results;
          }
        default:
          results.push(node);
          return results;
      }
    },
    [],
  );
  const dependencyNames =
    dependencies && dependencies.map((node) => parseDependencyNode(node, context));
  if (
    !isNamedTermNode(CONST_DIRECTIVE, instruction) ||
    !identifier ||
    !isIdentifierNode(identifier) ||
    !type ||
    !isTermNode(type) ||
    !dependencyNames ||
    !dependencyNames.every(Boolean) ||
    !initializer
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONST_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  registerInitializer(identifier, dependencyNames, node.location, context);
  return getTemplateElements(
    context.import(TEMPLATE, {
      $identifier: identifier,
      $type: type,
      $initializer: createBlockDirective({ elements: initializer, location: node.location }),
    }).module,
  );
}

function registerInitializer(identifier, dependencies, location, context) {
  const name = identifier.source;
  const initializers = context.globals.get(CONST_INITIALIZERS_GLOBAL) || new Map();
  if (initializers.has(name)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      location,
      source,
      `Duplicate ${CONST_DIRECTIVE} definition: ${formatSourceRange(source, node.location)}`,
    );
  }
  context.globals.set(
    CONST_INITIALIZERS_GLOBAL,
    initializers.set(name, { identifier, dependencies }),
  );
}

function isDependencyNode(node) {
  return isNamedInstructionNode(DEPENDS_ON_DIRECTIVE, node);
}

function parseDependencyNode(node, context) {
  const [instruction, identifier, ...varArgs] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(DEPENDS_ON_DIRECTIVE, instruction) ||
    !identifier ||
    !isIdentifierNode(identifier) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${DEPENDS_ON_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return identifier.source;
}

function getTemplateElements(template) {
  return template.statements;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}
