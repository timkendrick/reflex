// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import {
  createStructMethodNodes,
  createInlineStructDefinitionNodes,
  parseStructDefinition,
  STRUCT_DIRECTIVE,
} from './struct.mjs';
import {
  createInlineUnionDefinitionNodes,
  createUnionMethodNodes,
  parseUnionDefinition,
  UNION_DIRECTIVE,
} from './union.mjs';

export const CONSTRUCTOR_DIRECTIVE = '@constructor';

export default function constructorDirective(node, context) {
  if (!isNamedInstructionNode(CONSTRUCTOR_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONSTRUCTOR_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, root, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  const methods = root && createConstructorNodes(root, context);
  if (!isNamedTermNode(CONSTRUCTOR_DIRECTIVE, instruction) || !methods || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONSTRUCTOR_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return methods;
}

function createConstructorNodes(node, context) {
  if (isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const { identifier, fields } = parseStructDefinition(node, context);
    return [
      ...createStructMethodNodes(identifier, fields, null, node.location, context),
      ...createInlineStructDefinitionNodes(fields, context),
    ];
  } else if (isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const { identifier, variants } = parseUnionDefinition(node, context);
    return [
      ...createUnionMethodNodes(identifier, variants, null, node.location, context),
      ...createInlineUnionDefinitionNodes(variants, context),
    ];
  } else {
    return null;
  }
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
