// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const BLOCK_DIRECTIVE = '@block';

export default function blockDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, ...children] = elements;
  if (!isNamedTermNode(BLOCK_DIRECTIVE, instruction)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${BLOCK_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return children;
}

export function createBlockDirective({ elements, location }) {
  return NodeType.Instruction({
    instruction: BLOCK_DIRECTIVE,
    elements: [
      NodeType.Term({ source: BLOCK_DIRECTIVE, location }),
      NodeType.Whitespace({ source: ' ', location }),
      ...elements,
    ],
    location,
  });
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
