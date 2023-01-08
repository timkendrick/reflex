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
  const [instruction, identifier, type, dependencies, ...instructions] = node.elements.reduce(
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
    !dependencyNames.every(Boolean)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONST_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const initializer = parseInitializer(identifier, instructions, node.location, context);
  if (!initializer) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONST_DIRECTIVE} directive initializer: ${formatSourceRange(
        source,
        node.location,
      )}`,
    );
  }
  registerInitializer(identifier, type, dependencyNames, node.location, context);
  return getTemplateElements(
    context.import(TEMPLATE, {
      $identifier: identifier,
      $type: type,
      $default: getDefaultValue(type.source, node.location, context),
      $initializer: initializer,
    }).module,
  );
}

function getDefaultValue(type, location, context) {
  switch (type) {
    case 'i32':
      return NodeType.Instruction({
        instruction: 'i32.const',
        elements: [
          NodeType.Term({ source: 'i32.const', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: '-1', location }),
        ],
        location,
      });
    case 'i64':
      return NodeType.Instruction({
        instruction: 'i64.const',
        elements: [
          NodeType.Term({ source: 'i64.const', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: '-1', location }),
        ],
        location,
      });
    case 'f32':
      return NodeType.Instruction({
        instruction: 'f32.const',
        elements: [
          NodeType.Term({ source: 'f32.const', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: '0.0', location }),
        ],
        location,
      });
    case 'f64':
      return NodeType.Instruction({
        instruction: 'f64.const',
        elements: [
          NodeType.Term({ source: 'f64.const', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: '0.0', location }),
        ],
        location,
      });
    default: {
      const source = context.sources.get(context.path);
      throw new ParseError(
        location,
        source,
        `Invalid ${CONST_DIRECTIVE} directive type: ${formatSourceRange(
          source,
          location,
        )}`,
      );
    }
  }
}

function registerInitializer(identifier, type, dependencies, location, context) {
  const initializers = context.globals.get(CONST_INITIALIZERS_GLOBAL) || new Map();
  if (initializers.has(identifier.source)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      location,
      source,
      `Duplicate ${CONST_DIRECTIVE} definition: ${formatSourceRange(source, node.location)}`,
    );
  }
  context.globals.set(
    CONST_INITIALIZERS_GLOBAL,
    initializers.set(identifier.source, {
      identifier,
      type,
      initializer: NodeType.Term({
        source: getInitializerName(identifier),
        location: identifier.location,
      }),
      dependencies,
    }),
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

function parseInitializer(identifier, instructions, location, context) {
  const initializerBody = (parseFunctionInitializer(instructions, context) || instructions).flatMap(
    (node) => (context.transform ? context.transform(node, context) : [node]),
  );
  return NodeType.Instruction({
    instruction: 'func',
    elements: [
      NodeType.Term({ source: 'func', location }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Term({ source: getInitializerName(identifier), location }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Instruction({
        instruction: 'result',
        elements: [
          NodeType.Term({ source: 'result', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: 'i32', location }),
        ],
        location,
      }),
      NodeType.Whitespace({ source: '\n    ', location }),
      ...initializerBody,
    ],
    location,
  });
}

function getInitializerName(identifier) {
  return `${identifier.source}::initialize`;
}

function parseFunctionInitializer(elements, context) {
  const [initializer, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  if (!initializer || !isNamedInstructionNode('func', initializer) || varArgs.length > 0) {
    return null;
  }
  const [identifier, signature, ...body] = initializer.elements.reduce((results, node) => {
    results.push(...(results.length > 2 || !isNonFunctionalNode(node) ? [node] : []));
    return results;
  }, []);
  if (
    !identifier ||
    !isNamedTermNode('func', identifier) ||
    !signature ||
    !isResultNode('i32', signature)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      initializer.location,
      source,
      `Invalid ${CONST_DIRECTIVE} directive initializer: ${formatSourceRange(
        source,
        initializer.location,
      )}`,
    );
  }
  return body;
}

function isResultNode(resultType, node) {
  if (!isNamedInstructionNode('result', node)) return false;
  const [instruction, ...signature] = node.elements.filter((node) => !isNonFunctionalNode(node));
  if (!isNamedTermNode('result', instruction)) return false;
  const resultTypes = Array.isArray(resultType) ? resultType : [resultType];
  if (signature.length !== resultTypes.length) return false;
  return signature.every((node, index) => isTermNode(node) && node.source === resultTypes[index]);
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
