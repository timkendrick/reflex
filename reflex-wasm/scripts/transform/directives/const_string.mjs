// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/const_string.wat');

export const CONST_STRING_DIRECTIVE = '@const-string';

export default function constStringDirective(node, context) {
  const [instruction, identifier, value, ...varArgs] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(CONST_STRING_DIRECTIVE, instruction) ||
    !identifier ||
    !isTermNode(identifier) ||
    !value ||
    !isStringNode(value) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONST_STRING_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return getTemplateElements(
    context.import(TEMPLATE, {
      $identifier: identifier,
      $value: value,
    }).module,
  );
}

function getTemplateElements(template) {
  return template.statements;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}

function isStringNode(node) {
  return node.type === NodeType.String;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
