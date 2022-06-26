// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const EXPORT_DIRECTIVE = '@export';

export default function exportDirective(node, context) {
  const [instruction, identifier, ...value] = node.elements.reduce((results, node) => {
    results.push(
      ...(context.transform ? context.transform(node, context) : [node]).filter(
        (node) => !isNonFunctionalNode(node),
      ),
    );
    return results;
  }, []);
  if (!isNamedTermNode(EXPORT_DIRECTIVE, instruction) || !isIdentifierNode(identifier)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${EXPORT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  context.exports.set(identifier.source, value);
  return [];
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}
