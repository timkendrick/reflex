// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { createChildScope } from '../loader.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const LET_DIRECTIVE = '@let';

export default function letDirective(node, context) {
  const [instruction, identifier, value, ...instructions] = node.elements.reduce(
    (results, node) => {
      if (results.length < 3) {
        results.push(
          ...(context.transform ? context.transform(node, context) : [node]).filter(
            (node) => !isNonFunctionalNode(node),
          ),
        );
      } else if (!isNonFunctionalNode(node)) {
        results.push(node);
      }
      return results;
    },
    [],
  );
  if (!isNamedTermNode(LET_DIRECTIVE, instruction) || !isIdentifierNode(identifier) || !value) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${LET_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const childContext = createChildScope(context, { [identifier.source]: value });
  return instructions.flatMap((instruction) =>
    context.transform ? context.transform(instruction, childContext) : [instruction],
  );
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
