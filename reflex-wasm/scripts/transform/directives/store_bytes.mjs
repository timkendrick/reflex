// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const STORE_BYTES_DIRECTIVE = '@store_bytes';

export default function storeBytesDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, buffer, ...args] = elements.reduce((results, node) => {
    if (results.length >= 2 || !isNonFunctionalNode(node)) results.push(node);
    return results;
  }, []);
  if (!instruction || !buffer || !isNamedTermNode(STORE_BYTES_DIRECTIVE, instruction)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STORE_BYTES_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const [instructions, offset] = args.reduce(
    (result, arg) => {
      const [instructions, offset] = result;
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
                buffer,
                offset + index,
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
          return [instructions, offset + chars.length];
        case NodeType.Instruction:
          instructions.push(createStoreByteInstruction(buffer, offset, arg, null));
          instructions.push(
            NodeType.Whitespace({
              source: '\n',
              location: arg.location,
            }),
          );
          return [instructions, offset + 1];
        case NodeType.Program:
        case NodeType.Term:
          throw new ParseError(
            arg.location,
            source,
            `Invalid ${STORE_BYTES_DIRECTIVE} argument: ${formatSourceRange(source, arg.location)}`,
          );
      }
    },
    [[], 0],
  );
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
        source: `${offset}`,
        location: node.location,
      }),
    ],
    location: node.location,
  });
  instructions.unshift(
    NodeType.Instruction({
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
        buffer,
        NodeType.Whitespace({
          source: ' ',
          location: node.location,
        }),
        length,
      ],
      location: node.location,
    }),
    NodeType.Whitespace({
      source: '\n',
      location: node.location,
    }),
  );
  instructions.push(length);
  return instructions;
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

function createStoreByteInstruction(buffer, offset, value, comment, location) {
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
      buffer,
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

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
