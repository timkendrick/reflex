// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const INSTRUCTION_DIRECTIVE = '@instruction';

export default function instructionDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, instructionType, ...args] = elements.reduce((results, node) => {
    if (results.length > 2 || !isNonFunctionalNode(node)) results.push(node);
    return results;
  }, []);
  if (!isNamedTermNode(INSTRUCTION_DIRECTIVE, instruction) || !isTermNode(instructionType)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${INSTRUCTION_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Instruction({
      instruction: instructionType.source,
      elements: [instructionType, ...args],
      location: node.location,
    }),
  ];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}
