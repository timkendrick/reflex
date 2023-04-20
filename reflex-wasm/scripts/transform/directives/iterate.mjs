// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createBlockDirective } from './block.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/iterate.wat');

export const ITERATE_DIRECTIVE = '@iterate';

export default function iterateDirective(node, context) {
  const [
    instruction,
    labelIdentifier,
    sourceIdentifier,
    itemIdentifier,
    iteratorStateIdentifier,
    stateIdentifier,
    dependenciesIdentifier,
    ...body
  ] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .reduce((results, node) => {
      results.push(...(results.length < 7 && isNonFunctionalNode(node) ? [] : [node]));
      return results;
    }, []);
  if (
    !isNamedTermNode(ITERATE_DIRECTIVE, instruction) ||
    !labelIdentifier ||
    !isIdentifierNode(labelIdentifier) ||
    !sourceIdentifier ||
    !isIdentifierNode(sourceIdentifier) ||
    !itemIdentifier ||
    !isIdentifierNode(itemIdentifier) ||
    !iteratorStateIdentifier ||
    !isIdentifierNode(iteratorStateIdentifier) ||
    !stateIdentifier ||
    !isIdentifierNode(stateIdentifier) ||
    !dependenciesIdentifier ||
    !isIdentifierNode(dependenciesIdentifier)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${ITERATE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return getTemplateElements(
    context.import(TEMPLATE, {
      $label: labelIdentifier,
      $source: sourceIdentifier,
      $item: itemIdentifier,
      $iterator_state: iteratorStateIdentifier,
      $state: stateIdentifier,
      $dependencies: dependenciesIdentifier,
      $body: createBlockDirective({
        elements: body,
        location: node.location,
      }),
    }).module,
  );
}

function getTemplateElements(template) {
  return template.statements;
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
