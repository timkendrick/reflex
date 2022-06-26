// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements } from './list.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createBranchInstruction } from './branch.mjs';

export const SWITCH_DIRECTIVE = '@switch';

export default function switchDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, casesList, ...fallback] = elements.reduce((results, node) => {
    if (results.length >= 3 || !isNonFunctionalNode(node)) results.push(node);
    return results;
  }, []);
  const branches = getListElements(casesList);
  const cases =
    branches &&
    branches.map((branch) => {
      const pair = getListElements(branch);
      if (!pair) return pair;
      const [condition, consequent, ...varArgs] = pair;
      if (!condition || !consequent || varArgs.lengh > 0) return null;
      return [condition, consequent];
    });
  if (!isNamedTermNode(SWITCH_DIRECTIVE, instruction) || !cases) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${SWITCH_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const discriminant = cases.reduceRight(
    (inner, [condition, _consequent], index) => NodeType.Instruction({
      instruction: 'select',
      elements: [
        NodeType.Term({
          source: 'select',
          location: condition.location,
        }),
        NodeType.Whitespace({
          source: ' ',
          location: condition.location,
        }),
        NodeType.Instruction({
          instruction: 'i32.const',
          elements: [
            NodeType.Term({
              source: 'i32.const',
              location: condition.location,
            }),
            NodeType.Whitespace({
              source: ' ',
              location: condition.location,
            }),
            NodeType.Term({
              source: `${index}`,
              location: condition.location,
            }),
          ],
          location: condition.location,
        }),
        NodeType.Whitespace({
          source: '\n',
          location: condition.location,
        }),
        inner,
        NodeType.Whitespace({
          source: '\n',
          location: condition.location,
        }),
        condition,
      ]
    }),
    NodeType.Instruction({
      instruction: 'i32.const',
      elements: [
        NodeType.Term({
          source: 'i32.const',
          location: node.location,
        }),
        NodeType.Whitespace({
          source: ' ',
          location: node.location,
        }),
        NodeType.Term({
          source: '0xFFFFFFFF',
          location: node.location,
        }),
      ],
      location: node.location,
    })
  );
  return createBranchInstruction(
    discriminant,
    cases.map(([_condition, consequent]) => consequent),
    fallback,
    node.location,
  );
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
