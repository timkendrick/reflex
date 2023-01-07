// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createListDirective } from './list.mjs';
import {
  getStructConstructorFields,
  getStructFieldDataType,
  parseStructDefinition,
  STRUCT_DIRECTIVE,
} from './struct.mjs';
import { parseUnionDefinition, UNION_DIRECTIVE } from './union.mjs';

export const SIGNATURES_DIRECTIVE = '@signatures';

export default function signaturesDirective(node, context) {
  if (!isNamedInstructionNode(SIGNATURES_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${SIGNATURES_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, root, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  const signatures = root && parseConstructorSignatures(root, context);
  if (!isNamedTermNode(SIGNATURES_DIRECTIVE, instruction) || !signatures || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${SIGNATURES_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    createListDirective({
      elements: signatures.map(({ identifier, variant, args }) =>
        createListDirective({
          elements: [
            identifier,
            variant,
            createListDirective({
              elements: args.map(({ identifier, type }) =>
                createListDirective({
                  elements: [identifier, type],
                  location: node.location,
                }),
              ),
              location: node.location,
            }),
          ],
          location: node.location,
        }),
      ),
      location: node.location,
    }),
  ];
}

function parseConstructorSignatures(node, context) {
  if (isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const { identifier, fields } = parseStructDefinition(node, context);
    return parseStructConstructorSignatures(identifier, null, fields, node.location);
  } else if (isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const { identifier, variants } = parseUnionDefinition(node, context);
    return parseUnionConstructorSignatures(identifier, null, variants, node.location);
  } else {
    return null;
  }
}

function parseStructConstructorSignatures(identifier, variant, fields, location) {
  return [
    {
      identifier,
      variant: variant || identifier,
      args: getStructConstructorFields(fields).map((field) => ({
        identifier: field.name,
        type: getStructFieldDataType(field, location),
      })),
    },
  ];
}

function parseUnionConstructorSignatures(identifier, variant, variants, location) {
  return variants.flatMap((item) => {
    const childIdentifier = NodeType.Term({
      source: `${identifier.source}::${item.identifier.source.slice('$'.length)}`,
      location,
    });
    const childVariantIdentifier = variant
      ? NodeType.Term({
          source: `${variant.source}::${item.identifier.source.slice('$'.length)}`,
          location,
        })
      : item.identifier;
    switch (item.type) {
      case 'struct':
        return parseStructConstructorSignatures(
          childIdentifier,
          childVariantIdentifier,
          item.fields,
          location,
        );
      case 'union':
        return parseUnionConstructorSignatures(
          childIdentifier,
          childVariantIdentifier,
          item.variants,
          location,
        );
    }
  });
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
