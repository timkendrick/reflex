// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements } from './list.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const BRANCH_DIRECTIVE = '@branch';

export default function branchDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, discriminant, branchesList, ...fallback] = elements.reduce(
    (results, node) => {
      if (results.length >= 3 || !isNonFunctionalNode(node)) results.push(node);
      return results;
    },
    [],
  );
  const branches = getListElements(branchesList);
  if (!isNamedTermNode(BRANCH_DIRECTIVE, instruction) || !discriminant || !branches) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${BRANCH_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return createBranchInstruction(discriminant, branches, fallback, node.location);
}

export function createBranchInstruction(discriminant, branches, fallback, location) {
  const blocks = [
    ...branches.map((branch, index) => [
      `$BR${index}`,
      `${index}`,
      getListElements(branch) || [branch],
      branch.location,
    ]),
    ['$default', 'default', fallback, location],
  ];
  return [
    NodeType.Comment({
      source: `;; ----- ${BRANCH_DIRECTIVE} ----- \n`,
      location: location,
    }),
    ...blocks.reduce(
      (inner, [label, comment, instructions, location]) => [
        NodeType.Instruction({
          instruction: 'block',
          elements: [
            NodeType.Term({ source: 'block', location }),
            NodeType.Whitespace({ source: ' ', location }),
            NodeType.Term({ source: label, location }),
            NodeType.Whitespace({ source: ' ', location }),
            ...inner,
          ],
          location,
        }),
        NodeType.Whitespace({ source: '\n', location }),
        NodeType.Comment({ source: `(; ${BRANCH_DIRECTIVE} ${comment} ;)`, location }),
        ...instructions,
      ],
      [
        NodeType.Instruction({
          instruction: 'br_table',
          elements: [
            NodeType.Term({ source: 'br_table', location: discriminant.location }),
            ...blocks.flatMap(([label, comment, instructions, location]) => [
              NodeType.Whitespace({ source: ' ', location }),
              NodeType.Term({ source: label, location }),
            ]),
            NodeType.Whitespace({ source: '\n', location: discriminant.location }),
            NodeType.Comment({ source: `(; ${BRANCH_DIRECTIVE} condition ;)`, location }),
            NodeType.Whitespace({ source: '\n', location: discriminant.location }),
            discriminant,
          ],
          location: discriminant.location,
        }),
      ],
    ),
  ];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
