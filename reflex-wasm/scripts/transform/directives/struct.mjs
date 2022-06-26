// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createBlockDirective } from './block.mjs';
import { parseFieldDefinition } from './field.mjs';
import { createListDirective } from './list.mjs';
import {
  createUnionMethodNodes,
  createInlineUnionDefinitionNodes,
  getUnionSize,
} from './union.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/struct.wat');

export const STRUCT_DIRECTIVE = '@struct';

const POINTER_FIELD_SIZE = 4;
const ARRAY_FIELD_SIZE = 4 + 4;

export default function structDirective(node, context) {
  if (!isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STRUCT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const structDefinition = NodeType.Instruction({
    instruction: STRUCT_DIRECTIVE,
    elements,
    location: node.location,
  });
  if (!parseStructDefinition(structDefinition, context)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STRUCT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [structDefinition];
}

export function parseStructDefinition(node, context) {
  if (!isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STRUCT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const [instruction, identifier, ...members] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const fields = members.map((node) => parseFieldDefinition(node, context));
  if (
    !isNamedTermNode(STRUCT_DIRECTIVE, instruction) ||
    !isIdentifierNode(identifier) ||
    !fields.every(Boolean)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STRUCT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  if (fields.length > 1 && fields.slice(0, -1).some(({ type }) => type.type === 'repeated')) {
    const source = context.sources.get(context.path);
    const location = fields.find(({ type }) => type.type === 'repeated').location;
    throw new ParseError(
      location,
      source,
      `Invalid ${STRUCT_DIRECTIVE} directive: repeated fields only valid as final field: ${formatSourceRange(
        source,
        location,
      )}`,
    );
  }
  return { identifier, fields, members };
}

export function createStructMethodNodes(identifier, fields, delegate, location, context) {
  const [primitiveFields = [], structFields = [], unionFields = [], repeatedFields = []] =
    partitionMultipleBy(createFieldTemplateValues(fields), ({ field }) => {
      const fieldType = field.type;
      switch (fieldType.type) {
        case 'primitive':
        case 'reference':
          return 0;
        case 'struct':
          return 1;
        case 'union':
          return 2;
        case 'repeated':
          return 3;
        default:
          throw new Error(`Invalid field type: ${fieldType.type}`);
      }
    });
  return getTemplateElements(
    context.import(TEMPLATE, {
      $struct_name: identifier,
      $struct_size: createNumericLiteralNode(getStructSize(fields), location),
      $delegate: delegate,
      $primitive_fields: createListDirective({
        elements: primitiveFields.map(({ node }) => node),
        location: location,
      }),
      $inline_fields: createListDirective({
        elements: [...structFields, ...unionFields].map(({ node }) => node),
        location: location,
      }),
      $repeated_fields: createListDirective({
        elements: repeatedFields.map(({ node }) => node),
        location: location,
      }),
    }).module,
  );
}

export function createInlineStructDefinitionNodes(fields, context) {
  return fields
    .map(({ type }) => type)
    .flatMap(function getFieldTypeNodes(fieldType) {
      switch (fieldType.type) {
        case 'repeated':
          return getFieldTypeNodes(fieldType.target);
        case 'struct':
          return [
            ...createStructMethodNodes(
              fieldType.identifier,
              fieldType.fields,
              null,
              fieldType.location,
              context,
            ),
            ...createInlineStructDefinitionNodes(fieldType.fields, context),
          ];
        case 'union':
          return [
            ...createUnionMethodNodes(
              fieldType.identifier,
              fieldType.variants,
              null,
              fieldType.location,
              context,
            ),
            ...createInlineUnionDefinitionNodes(fieldType.variants, context),
          ];
        default:
          return [];
      }
    });
}

function createFieldTemplateValues(fields) {
  return fields.reduce(
    ({ fields, offset }, field) => {
      const { name, location, exports } = field;
      const size = getStructFieldSize(field);
      const dataType = getStructFieldDataType(field, location);
      fields.push({
        field,
        node: createListDirective({
          elements: [
            name,
            createNumericLiteralNode(offset, location),
            dataType,
            createNumericLiteralNode(getStructFieldItemSize(field), location),
            exports.length > 1
              ? createBlockDirective(exports, location)
              : exports.length > 0
              ? exports[0]
              : NodeType.Term({ source: '', location: null }),
          ],
          location,
        }),
      });
      return {
        fields,
        offset: offset + size,
      };
    },
    { fields: [], offset: 0 },
  ).fields;
}

export function getStructSize(fields) {
  return fields.reduce((result, field) => result + getStructFieldSize(field), 0);
}

export function getStructConstructorFields(fields) {
  return fields.filter((field) => {
    const fieldType = field.type;
    switch (fieldType.type) {
      case 'primitive':
      case 'reference':
        return true;
      case 'struct':
      case 'union':
      case 'repeated':
        return false;
    }
  });
}

export function getStructFieldDataType(field, location) {
  const fieldType = field.type;
  switch (fieldType.type) {
    case 'primitive':
      return fieldType.target;
    case 'struct':
    case 'union':
    case 'reference':
    case 'repeated':
      return NodeType.Term({ source: 'i32', location });
    default:
      throw new Error(`Invalid field type: ${fieldType.type}`);
  }
}

export function getStructFieldSize(field) {
  const fieldType = field.type;
  return getStructFieldTypeSize(fieldType)
}

export function getStructFieldTypeSize(fieldType) {
  switch (fieldType.type) {
    case 'primitive':
      return fieldType.size;
    case 'reference':
      return POINTER_FIELD_SIZE;
    case 'repeated':
      return ARRAY_FIELD_SIZE;
    case 'struct':
      return getStructSize(fieldType.fields);
    case 'union':
      return getUnionSize(fieldType.variants);
    default:
      throw new Error(`Invalid field type: ${fieldType.type}`);
  }
}

export function getStructFieldItemSize(field) {
  const fieldType = field.type;
  return getStructFieldTypeItemSize(fieldType);
}

export function getStructFieldTypeItemSize(fieldType) {
  switch (fieldType.type) {
    case 'repeated': {
      const innerType = fieldType.target;
      switch (innerType.type) {
        case 'primitive':
          return innerType.size;
        case 'reference':
          return POINTER_FIELD_SIZE;
        case 'struct':
          return getStructSize(innerType.fields);
        case 'union':
          return getUnionSize(innerType.variants);
        default:
          throw new Error(`Invalid repeated field type: ${fieldType.type}`);
      }
    }
    default:
      return getStructFieldTypeSize(fieldType);
  }
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

function isTermNode(node) {
  return node.type === NodeType.Term;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function partitionMultipleBy(items, selector) {
  return items.reduce((results, item) => {
    const value = selector(item);
    if (results.length < value + 1) {
      results.push(...Array.from({ length: value + 1 - results.length }, () => []));
    }
    results[value].push(item);
    return results;
  }, []);
}
