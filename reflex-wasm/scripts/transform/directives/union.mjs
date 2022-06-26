// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createListDirective } from './list.mjs';
import {
  createInlineStructDefinitionNodes,
  createStructMethodNodes,
  getStructConstructorFields,
  getStructFieldDataType,
  getStructSize,
  parseStructDefinition,
  STRUCT_DIRECTIVE,
} from './struct.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/union.wat');

export const UNION_DIRECTIVE = '@union';

export const UNION_DISCRIMINANT_SIZE = 4;

export default function unionDirective(node, context) {
  if (!isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const unionDefinition = NodeType.Instruction({
    instruction: UNION_DIRECTIVE,
    elements,
    location: node.location,
  });
  if (!parseUnionDefinition(unionDefinition, context)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [unionDefinition];
}

export function createUnionMethodNodes(identifier, variants, delegate, location, context) {
  return getTemplateElements(
    context.import(TEMPLATE, {
      $union_name: identifier,
      $union_size: createNumericLiteralNode(getUnionSize(variants), location),
      $delegate: delegate,
      $variants: createListDirective({
        elements: variants
          .map((variant, index) => ({ variant, index }))
          .flatMap(({ variant, index }) => {
            const constructors = getUnionVariantConstructorFields(variant, location);
            return constructors.map(({ identifier, constructorFields }) =>
              createListDirective(
                {
                  elements: [
                    identifier,
                    createNumericLiteralNode(index, location),
                    createListDirective({
                      elements: constructorFields,
                      location,
                    }),
                  ],
                  location,
                },
                location,
              ),
            );
          }),
        location,
      }),
    }).module,
  );
}

export function createInlineUnionDefinitionNodes(variants, context) {
  return variants.flatMap((variant) => {
    switch (variant.type) {
      case 'struct':
        return [
          ...createStructMethodNodes(
            variant.identifier,
            variant.fields,
            null,
            variant.location,
            context,
          ),
          ...createInlineStructDefinitionNodes(variant.fields, context),
        ];
      case 'union':
        return [
          ...createUnionMethodNodes(
            variant.identifier,
            variant.variants,
            null,
            variant.location,
            context,
          ),
          ...createInlineUnionDefinitionNodes(variant.variants, context),
        ];
      default:
        throw new Error(`Invalid union variant type: ${variant.type}`);
    }
  });
}

function getUnionVariantConstructorFields(variant, location) {
  switch (variant.type) {
    case 'struct':
      return [
        {
          identifier: variant.identifier,
          constructorFields: getStructConstructorFields(variant.fields).map((field) =>
            createListDirective({
              elements: [field.name, getStructFieldDataType(field, location)],
              location,
            }),
          ),
        },
      ];
    case 'union':
      return variant.variants.flatMap((childVariant) =>
        getUnionVariantConstructorFields(
          {
            ...childVariant,
            identifier: NodeType.Term({
              source: `${variant.identifier.source}::${childVariant.identifier.source.slice(
                '$'.length,
              )}`,
              location,
            }),
          },
          location,
        ),
      );
  }
}

export function getUnionSize(variants) {
  return (
    UNION_DISCRIMINANT_SIZE +
    Math.max(
      ...variants.map((variant) => {
        switch (variant.type) {
          case 'struct':
            return getStructSize(variant.fields);
          case 'union':
            return getUnionSize(variant.variants);
          default:
            throw new Error(`Invalid union variant type: ${variant.type}`);
        }
      }),
    )
  );
}

export function parseUnionDefinition(node, context) {
  if (!isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const [instruction, identifier, ...members] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const variants = members.map((node) => parseVariantDefinition(node, context));
  if (
    !isNamedTermNode(UNION_DIRECTIVE, instruction) ||
    !isIdentifierNode(identifier) ||
    !variants.every(Boolean) ||
    variants.length == 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return { identifier, variants, members };
}

function parseVariantDefinition(node, context) {
  return parseStructVariant(node, context) || parseUnionVariant(node, context);
}

function parseStructVariant(node, context) {
  if (!isNamedInstructionNode(STRUCT_DIRECTIVE, node)) return null;
  const { identifier, fields } = parseStructDefinition(node, context);
  return {
    type: 'struct',
    identifier,
    fields,
    location: node.location,
  };
}

function parseUnionVariant(node, context) {
  if (!isNamedInstructionNode(UNION_DIRECTIVE, node)) return null;
  const { identifier, variants } = parseUnionDefinition(node, context);
  return {
    type: 'union',
    identifier,
    variants,
    location: node.location,
  };
}

function getTemplateElements(template) {
  return template.statements;
}

function createNumericLiteralNode(value, location) {
  return NodeType.Term({ source: `${value}`, location });
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

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}
