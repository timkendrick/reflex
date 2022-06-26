// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createListDirective } from './list.mjs';
import {
  createStructMethodNodes,
  getStructConstructorFields,
  getStructFieldDataType,
  parseStructDefinition,
  STRUCT_DIRECTIVE,
} from './struct.mjs';
import { createUnionMethodNodes, parseUnionDefinition, UNION_DIRECTIVE } from './union.mjs';

export const DELEGATE_DIRECTIVE = '@delegate';

export default function delegateDirective(node, context) {
  if (!isNamedInstructionNode(DELEGATE_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${DELEGATE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, prefix, root, selector, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const methods =
    isTermNode(prefix) &&
    (isNamedInstructionNode(STRUCT_DIRECTIVE, root) ||
      isNamedInstructionNode(UNION_DIRECTIVE, root)) &&
    selector
      ? createDelegateMethodNodes(root, prefix, selector, node.location, context)
      : null;
  if (!isNamedTermNode(DELEGATE_DIRECTIVE, instruction) || !methods || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${DELEGATE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return methods;
}

function createDelegateMethodNodes(node, prefix, selector, location, context) {
  const delegate = NodeType.Instruction({
    instruction: 'local.set',
    elements: [
      NodeType.Term({ source: 'local.set', location }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Term({ source: '0', location }),
      NodeType.Whitespace({ source: ' ', location }),
      selector,
    ],
    location,
  });
  if (isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const { identifier, fields } = parseStructDefinition(node, context);
    return createStructMethodNodes(
      NodeType.Term({
        source: `${prefix.source}::${identifier.source.slice('$'.length)}`,
        location: identifier.location,
      }),
      fields,
      delegate,
      node.location,
      context,
    );
  } else if (isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const { identifier, variants } = parseUnionDefinition(node, context);
    return createUnionMethodNodes(
      NodeType.Term({
        source: `${prefix.source}::${identifier.source.slice('$'.length)}`,
        location: identifier.location,
      }),
      variants,
      delegate,
      node.location,
      context,
    );
  } else {
    return null;
  }
}

function parseStructConstructorSignatures(identifier, fields, location) {
  return [
    {
      identifier,
      args: getStructConstructorFields(fields).map((field) => ({
        identifier: field.name,
        type: getStructFieldDataType(field, location),
      })),
    },
  ];
}

function parseUnionConstructorSignatures(identifier, variants, location) {
  return variants.flatMap((variant) => {
    const childIdentifier = NodeType.Term({
      source: `${identifier.source}::${variant.identifier.source.slice('$'.length)}`,
      location,
    });
    switch (variant.type) {
      case 'struct':
        return parseStructConstructorSignatures(childIdentifier, variant.fields, location);
      case 'union':
        return parseUnionConstructorSignatures(childIdentifier, variant.variants, location);
    }
  });
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}
