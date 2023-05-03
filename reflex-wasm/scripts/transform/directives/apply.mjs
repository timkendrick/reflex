// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { createChildScope } from '../loader.mjs';
import { NodeType } from '../parser.mjs';
import { TEMPLATE_DIRECTIVE } from './template.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const APPLY_DIRECTIVE = '@apply';

export default function applyDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, template, ...args] = elements.filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(APPLY_DIRECTIVE, instruction) ||
    !isNamedInstructionNode(template, TEMPLATE_DIRECTIVE) ||
    args.length !== template.elements.length - 1
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${APPLY_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const argIdentifiers = template.elements.slice(0, -1);
  const body = template.elements[template.elements.length - 1];
  const argValues = Object.fromEntries(
    argIdentifiers.map((identifier, index) => [identifier.source, args[index]]),
  );
  const childContext = createChildScope(context, argValues);
  return context.transform ? context.transform(body, childContext) : [body];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isNamedInstructionNode(node, instruction) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}
