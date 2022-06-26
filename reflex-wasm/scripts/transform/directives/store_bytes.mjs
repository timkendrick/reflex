// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const STORE_BYTES_DIRECTIVE = '@store-bytes';

export default function storeBytesDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, targetIdentifier, ...args] = elements.reduce((results, node) => {
    if (results.length >= 2 || !isNonFunctionalNode(node)) results.push(node);
    return results;
  }, []);
  if (
    !isNamedTermNode(STORE_BYTES_DIRECTIVE, instruction) ||
    !targetIdentifier ||
    !isIdentifierNode(targetIdentifier)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STORE_BYTES_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const target = NodeType.Instruction({
    instruction: 'local.get',
    elements: [
      NodeType.Term({ source: 'local.get', location: targetIdentifier.location }),
      NodeType.Whitespace({ source: ' ', location: targetIdentifier.location }),
      targetIdentifier,
    ],
    location: targetIdentifier.location,
  });
  const [instructions, output] = args.reduce(
    (result, arg) => {
      const [instructions, output] = result;
      switch (arg.type) {
        case NodeType.Comment:
        case NodeType.Whitespace:
          instructions.push(arg);
          return result;
        case NodeType.String:
          const chars = JSON.parse(arg.source).split('');
          instructions.push(
            ...chars.flatMap((char, index) => [
              createStoreByteInstruction(
                target,
                output.length + index,
                createCharInstruction(char, arg.location),
                JSON.stringify(char),
                arg.location,
              ),
              NodeType.Whitespace({
                source: '\n',
                location: arg.location,
              }),
            ]),
          );
          return [instructions, output + chars.join('')];
        case NodeType.Instruction:
          instructions.push(createStoreByteInstruction(target, output.length, arg, null));
          instructions.push(
            NodeType.Whitespace({
              source: '\n',
              location: arg.location,
            }),
          );
          return [instructions, output.length + '?'];
        case NodeType.Program:
        case NodeType.Term: {
          const source = context.sources.get(context.path);
          throw new ParseError(
            arg.location,
            source,
            `Invalid ${STORE_BYTES_DIRECTIVE} argument: ${formatSourceRange(source, arg.location)}`,
          );
        }
      }
    },
    [[], ''],
  );
  const allocation = NodeType.Instruction({
    instruction: 'call',
    elements: [
      NodeType.Term({
        source: 'call',
        location: node.location,
      }),
      NodeType.Whitespace({
        source: ' ',
        location: node.location,
      }),
      NodeType.Term({
        source: '$Allocator::extend',
        location: node.location,
      }),
      NodeType.Whitespace({
        source: ' ',
        location: node.location,
      }),
      target,
      NodeType.Whitespace({
        source: ' ',
        location: node.location,
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
            source: `${output.length}`,
            location: node.location,
          }),
        ],
        location: node.location,
      }),
    ],
    location: node.location,
  });
  const length = NodeType.Instruction({
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
        source: `${output.length}`,
        location: node.location,
      }),
    ],
    location: node.location,
  });
  return [
    NodeType.Comment({
      source: `;; ----- Begin ${STORE_BYTES_DIRECTIVE}: ${JSON.stringify(output)} -----\n`,
      location: node.location,
    }),
    allocation,
    NodeType.Whitespace({
      source: '\n',
      location: node.location,
    }),
    ...instructions,
    length,
    NodeType.Whitespace({
      source: '\n',
      location: node.location,
    }),
    NodeType.Comment({
      source: `;; ----- End ${STORE_BYTES_DIRECTIVE}: ${JSON.stringify(output)} -----\n`,
      location: node.location,
    }),
  ];
}

function createCharInstruction(char, location) {
  return NodeType.Instruction({
    instruction: 'i32.const',
    elements: [
      NodeType.Term({
        source: 'i32.const',
        location,
      }),
      NodeType.Whitespace({
        source: ' ',
        location,
      }),
      NodeType.Term({
        source: `${char.charCodeAt(0)}`,
        location,
      }),
    ],
    location,
  });
}

function createStoreByteInstruction(target, offset, value, comment, location) {
  return NodeType.Instruction({
    instruction: 'i32.store8',
    elements: [
      NodeType.Term({
        source: 'i32.store8',
        location,
      }),
      NodeType.Whitespace({
        source: ' ',
        location,
      }),
      NodeType.Term({
        source: `offset=${offset}`,
        location,
      }),
      NodeType.Whitespace({
        source: ' ',
        location,
      }),
      target,
      NodeType.Whitespace({
        source: ' ',
        location,
      }),
      ...(comment ? [NodeType.Comment({ source: `(; ${comment} ;)`, location })] : []),
      value,
    ],
    location,
  });
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
