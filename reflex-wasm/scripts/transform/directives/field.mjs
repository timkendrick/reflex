// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { parseStructDefinition, STRUCT_DIRECTIVE } from './struct.mjs';
import { parseUnionDefinition, UNION_DIRECTIVE } from './union.mjs';

export const FIELD_DIRECTIVE = '@field';
export const REPEATED_DIRECTIVE = '@repeated';
export const REFERENCE_TYPE_NODE_DIRECTIVE = '@ref';
export const OPTIONAL_MODIFIER_DIRECTIVE = '@optional';

export default function fieldDirective(node, context) {
  if (!isNamedInstructionNode(FIELD_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${FIELD_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const fieldDefinition = NodeType.Instruction({
    instruction: FIELD_DIRECTIVE,
    elements,
    location: node.location,
  });
  if (!parseFieldDefinition(fieldDefinition, context)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${FIELD_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [fieldDefinition];
}

export function parseFieldDefinition(node, context) {
  const [instruction, fieldIdentifier, ...typeNodes] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const fieldType = parseFieldType(typeNodes, context);
  if (
    !isNamedTermNode(FIELD_DIRECTIVE, instruction) ||
    !isIdentifierNode(fieldIdentifier) ||
    !fieldType
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${FIELD_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const { type, exports } = fieldType;
  return {
    name: fieldIdentifier,
    type,
    exports,
    location: node.location,
  };
}

function parseFieldType(nodes, context) {
  if (nodes.length === 0) return null;
  const type = parseDataType(nodes[nodes.length - 1], context);
  if (!type) return null;
  const exports = parseFieldExports(nodes.slice(0, -1));
  if (!exports) return null;
  return { type, exports };
}

function parseFieldExports(nodes) {
  return nodes.reduce((result, node) => {
    if (result === null) return null;
    if (isNamedInstructionNode('export', node)) {
      result.push(node);
      return result;
    }
    return null;
  }, []);
}

function parseDataType(node, context) {
  return (
    parsePrimitiveTypeDefinitionNode(node) ||
    parseInlineStructDefinitionNode(node, context) ||
    parseInlineUnionDefinitionNode(node, context) ||
    parseReferenceTypeDefinitionNode(node, context) ||
    parseRepeatedTypeDefinitionNode(node, context)
  );
}

function parsePrimitiveTypeDefinitionNode(node) {
  if (!isTermNode(node)) return null;
  switch (node.source) {
    case 'i32':
      return { type: 'primitive', target: node, size: 4 };
    case 'i64':
      return { type: 'primitive', target: node, size: 8 };
    case 'f32':
      return { type: 'primitive', target: node, size: 4 };
    case 'f64':
      return { type: 'primitive', target: node, size: 8 };
    default:
      return null;
  }
}

function parseInlineStructDefinitionNode(node, context) {
  if (!isNamedInstructionNode(STRUCT_DIRECTIVE, node)) return null;
  const { identifier, fields } = parseStructDefinition(node, context);
  return {
    type: 'struct',
    identifier,
    fields,
    location: node.location,
  };
}

function parseInlineUnionDefinitionNode(node, context) {
  if (!isNamedInstructionNode(UNION_DIRECTIVE, node)) return null;
  const { identifier, variants } = parseUnionDefinition(node, context);
  return {
    type: 'union',
    identifier,
    variants,
    location: node.location,
  };
}

function parseReferenceTypeDefinitionNode(node, context) {
  if (!isNamedInstructionNode(REFERENCE_TYPE_NODE_DIRECTIVE, node)) return null;
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, target, ...modifiers] = elements.filter((node) => !isNonFunctionalNode(node));
  const referenceModifiers = modifiers.reduce(
    (result, node) => {
      if (isNamedTermNode(OPTIONAL_MODIFIER_DIRECTIVE, node)) {
        return result.optional ? null : { ...result, optional: true };
      } else {
        return null;
      }
    },
    { optional: false },
  );
  if (
    !isNamedTermNode(REFERENCE_TYPE_NODE_DIRECTIVE, instruction) ||
    !isIdentifierNode(target) ||
    !referenceModifiers
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${REFERENCE_TYPE_NODE_DIRECTIVE} directive: ${formatSourceRange(
        source,
        node.location,
      )}`,
    );
  }
  const { optional } = referenceModifiers;
  return { type: 'reference', target, optional };
}

function parseRepeatedTypeDefinitionNode(node, context) {
  if (!isNamedInstructionNode(REPEATED_DIRECTIVE, node)) return null;
  const [instruction, target, ...varArgs] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const targetType =
    target &&
    (parsePrimitiveTypeDefinitionNode(target) ||
      parseInlineStructDefinitionNode(target, context) ||
      parseInlineUnionDefinitionNode(target, context) ||
      parseReferenceTypeDefinitionNode(target, context));
  if (!isNamedTermNode(REPEATED_DIRECTIVE, instruction) || !targetType || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${REPEATED_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return { type: 'repeated', target: targetType };
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

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}
