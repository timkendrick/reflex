// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createBranchInstruction } from './branch.mjs';
import { createListDirective } from './list.mjs';
import {
  parseStructDefinition,
  STRUCT_DIRECTIVE,
  getStructFieldSize,
  getStructFieldItemSize,
  getStructFieldTypeItemSize,
} from './struct.mjs';
import {
  getUnionSize,
  parseUnionDefinition,
  UNION_DIRECTIVE,
  UNION_DISCRIMINANT_SIZE,
} from './union.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE_EQUALS_FIELD_PRIMITIVE = path.join(
  __dirname,
  '../templates/derive/equals_field_primitive.wat',
);
const TEMPLATE_EQUALS_FIELD_INLINE = path.join(
  __dirname,
  '../templates/derive/equals_field_inline.wat',
);
const TEMPLATE_EQUALS_FIELD_REFERENCE = path.join(
  __dirname,
  '../templates/derive/equals_field_reference.wat',
);
const TEMPLATE_EQUALS_FIELD_OPTIONAL_REFERENCE = path.join(
  __dirname,
  '../templates/derive/equals_field_optional_reference.wat',
);
const TEMPLATE_EQUALS_FIELD_REPEATED = path.join(
  __dirname,
  '../templates/derive/equals_field_repeated.wat',
);
const TEMPLATE_EQUALS_STRUCT = path.join(__dirname, '../templates/derive/equals_struct.wat');
const TEMPLATE_EQUALS_UNION = path.join(__dirname, '../templates/derive/equals_union.wat');
const TEMPLATE_HASH_FIELD_PRIMITIVE = path.join(
  __dirname,
  '../templates/derive/hash_field_primitive.wat',
);
const TEMPLATE_HASH_FIELD_INLINE = path.join(
  __dirname,
  '../templates/derive/hash_field_inline.wat',
);
const TEMPLATE_HASH_FIELD_REFERENCE = path.join(
  __dirname,
  '../templates/derive/hash_field_reference.wat',
);
const TEMPLATE_HASH_FIELD_OPTIONAL_REFERENCE = path.join(
  __dirname,
  '../templates/derive/hash_field_optional_reference.wat',
);
const TEMPLATE_HASH_FIELD_REPEATED = path.join(
  __dirname,
  '../templates/derive/hash_field_repeated.wat',
);
const TEMPLATE_HASH_STRUCT = path.join(__dirname, '../templates/derive/hash_struct.wat');
const TEMPLATE_HASH_UNION = path.join(__dirname, '../templates/derive/hash_union.wat');

export const DERIVE_DIRECTIVE = '@derive';

export default function deriveDirective(node, context) {
  if (!isNamedInstructionNode(DERIVE_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${DERIVE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, method, root, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const methodName = method && parseDeriveMethodName(method);
  const typeDefinition = root && parseTypeDefinition(root, context);
  if (
    !isNamedTermNode(DERIVE_DIRECTIVE, instruction) ||
    !methodName ||
    !typeDefinition ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${DERIVE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  switch (methodName) {
    case 'size':
      return createDeriveSizeMethodNodes(typeDefinition, node.location);
    case 'equals':
      return createDeriveEqualsMethodNodes(typeDefinition, node.location, context);
    case 'hash':
      return createDeriveHashMethodNodes(typeDefinition, node.location, context);
    default:
      throw new Error(`Unknown derive type: ${methodName}`);
  }
}

function createDeriveSizeMethodNodes(typeDefinition, location) {
  return [
    NodeType.Instruction({
      instruction: 'func',
      elements: [
        NodeType.Term({ source: 'func', location }),
        NodeType.Whitespace({ source: ' ', location }),
        NodeType.Term({ source: `${typeDefinition.identifier.source}::traits::size`, location }),
        NodeType.Whitespace({ source: ' ', location }),
        NodeType.Instruction({
          instruction: 'param',
          elements: [
            NodeType.Term({ source: 'param', location }),
            NodeType.Whitespace({ source: ' ', location }),
            NodeType.Term({ source: 'i32', location }),
          ],
          location,
        }),
        NodeType.Whitespace({ source: ' ', location }),
        NodeType.Instruction({
          instruction: 'result',
          elements: [
            NodeType.Term({ source: 'result', location }),
            NodeType.Whitespace({ source: ' ', location }),
            NodeType.Term({ source: 'i32', location }),
          ],
          location,
        }),
        NodeType.Whitespace({ source: '\n', location }),
        ...createDeriveSizeNodes(typeDefinition, location),
      ],
      location,
    }),
  ];
}

function createDeriveSizeNodes(typeDefinition, location) {
  switch (typeDefinition.type) {
    case 'struct':
      return createDeriveStructSizeNodes(
        typeDefinition.identifier,
        typeDefinition.fields,
        location,
      );
    case 'union':
      return createDeriveUnionSizeNodes(
        typeDefinition.identifier,
        typeDefinition.variants,
        location,
      );
    default:
      throw new Error(`Unknown type definition type: ${typeDefinition.type}`);
  }
}

function createDeriveStructSizeNodes(identifier, fields, location) {
  const { staticSize, dynamicNodes } = fields.reduce(
    ({ staticSize, dynamicNodes }, field) => {
      const fieldType = field.type;
      switch (fieldType.type) {
        case 'primitive':
        case 'reference':
          return {
            staticSize: staticSize + getStructFieldSize(field),
            dynamicNodes,
          };
        case 'repeated':
          return {
            staticSize: staticSize + getStructFieldSize(field),
            dynamicNodes: [
              ...dynamicNodes,
              NodeType.Instruction({
                instruction: 'i32.mul',
                elements: [
                  NodeType.Term({ source: 'i32.mul', location }),
                  NodeType.Whitespace({ source: ' ', location }),
                  NodeType.Instruction({
                    instruction: 'i32.const',
                    elements: [
                      NodeType.Term({ source: 'i32.const', location }),
                      NodeType.Whitespace({ source: ' ', location }),
                      NodeType.Term({ source: `${getStructFieldItemSize(field)}`, location }),
                    ],
                    location,
                  }),
                  NodeType.Whitespace({ source: ' ', location }),
                  NodeType.Instruction({
                    instruction: 'call',
                    elements: [
                      NodeType.Term({ source: 'call', location }),
                      NodeType.Whitespace({ source: ' ', location }),
                      NodeType.Term({
                        source: `${identifier.source}::get::${field.name.source.slice(
                          '$'.length,
                        )}::capacity`,
                        location,
                      }),
                      NodeType.Whitespace({ source: ' ', location }),
                      NodeType.Instruction({
                        instruction: 'local.get',
                        elements: [
                          NodeType.Term({ source: 'local.get', location }),
                          NodeType.Whitespace({ source: ' ', location }),
                          NodeType.Term({ source: '0', location }),
                        ],
                        location,
                      }),
                    ],
                    location,
                  }),
                ],
                location,
              }),
            ],
          };
        case 'struct':
        case 'union':
          return {
            staticSize,
            dynamicNodes: [
              ...dynamicNodes,
              NodeType.Instruction({
                instruction: 'call',
                elements: [
                  NodeType.Term({ source: 'call', location }),
                  NodeType.Whitespace({ source: ' ', location }),
                  NodeType.Term({
                    source: `${fieldType.identifier.source}::traits::size`,
                    location,
                  }),
                  NodeType.Whitespace({ source: ' ', location }),
                  NodeType.Instruction({
                    instruction: 'call',
                    elements: [
                      NodeType.Term({ source: 'call', location }),
                      NodeType.Whitespace({ source: ' ', location }),
                      NodeType.Term({
                        source: `${identifier.source}::pointer::${field.name.source.slice(
                          '$'.length,
                        )}`,
                        location,
                      }),
                      NodeType.Whitespace({ source: ' ', location }),
                      NodeType.Instruction({
                        instruction: 'local.get',
                        elements: [
                          NodeType.Term({ source: 'local.get', location }),
                          NodeType.Whitespace({ source: ' ', location }),
                          NodeType.Term({ source: '0', location }),
                        ],
                        location,
                      }),
                    ],
                    location,
                  }),
                ],
                location,
              }),
            ],
          };
        default:
          throw new Error(`Unknown field type: ${field.type}`);
      }
    },
    { staticSize: 0, dynamicNodes: [] },
  );
  return [
    dynamicNodes.reduce(
      (inner, node) =>
        NodeType.Instruction({
          instruction: 'i32.add',
          elements: [
            NodeType.Term({ source: 'i32.add', location }),
            NodeType.Whitespace({ source: '\n', location }),
            inner,
            NodeType.Whitespace({ source: '\n', location }),
            node,
          ],
          location,
        }),
      NodeType.Instruction({
        instruction: 'i32.const',
        elements: [
          NodeType.Term({ source: 'i32.const', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: `${staticSize}`, location }),
        ],
        location,
      }),
    ),
  ];
}

function createDeriveUnionSizeNodes(identifier, variants, location) {
  const staticSizeNode = NodeType.Instruction({
    instruction: 'i32.const',
    elements: [
      NodeType.Term({ source: 'i32.const', location }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Term({ source: `${getUnionSize(variants)}`, location }),
    ],
    location,
  });
  if (!isUnsizedUnion(variants)) {
    return [staticSizeNode];
  }
  const discriminant = NodeType.Instruction({
    instruction: 'call',
    elements: [
      NodeType.Term({ source: 'call', location }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Term({
        source: `${identifier.source}::get::type`,
        location,
      }),
      NodeType.Whitespace({ source: ' ', location }),
      NodeType.Instruction({
        instruction: 'local.get',
        elements: [
          NodeType.Term({ source: 'local.get', location }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Term({ source: '0', location }),
        ],
        location,
      }),
    ],
    location,
  });
  const branches = variants
    .map((variant) => {
      switch (variant.type) {
        case 'struct':
          return createDeriveStructSizeNodes(variant.identifier, variant.fields, location);
        case 'union':
          return createDeriveUnionSizeNodes(variant.identifier, variant.variants, location);
        default:
          throw new Error(`Unknown union variant type: ${variant.type}`);
      }
    })
    .map((branch) =>
      createListDirective({
        elements: [
          ...branch,
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Instruction({
            instruction: 'i32.add',
            elements: [
              NodeType.Term({ source: 'i32.add', location }),
              NodeType.Whitespace({ source: ' ', location }),
              NodeType.Instruction({
                source: 'i32.const',
                elements: [
                  NodeType.Term({ source: 'i32.const', location }),
                  NodeType.Whitespace({ source: ' ', location }),
                  NodeType.Term({ source: `${UNION_DISCRIMINANT_SIZE}`, location }),
                ],
              }),
            ],
            location,
          }),
          NodeType.Whitespace({ source: ' ', location }),
          NodeType.Instruction({
            instruction: 'return',
            elements: [NodeType.Term({ source: 'return', location })],
            location,
          }),
        ],
        location,
      }),
    );
  return createBranchInstruction(discriminant, branches, [staticSizeNode], location);
}

function isUnsizedUnion(variants) {
  return variants.some((variant) => {
    switch (variant.type) {
      case 'struct':
        return isUnsizedStruct(variant.fields);
      case 'union':
        return isUnsizedUnion(variant.variants);
      default:
        throw new Error(`Unknown union variant type: ${variant.type}`);
    }
  });
}

function isUnsizedStruct(fields) {
  return fields.some((field) => {
    const fieldType = field.type;
    return fieldType.type === 'repeated';
  });
}

function createDeriveEqualsMethodNodes(typeDefinition, location, context) {
  switch (typeDefinition.type) {
    case 'struct':
      return createDeriveStructEqualsMethodNodes(
        typeDefinition.identifier,
        typeDefinition.fields,
        location,
        context,
      );
    case 'union':
      return createDeriveUnionEqualsMethodNodes(
        typeDefinition.identifier,
        typeDefinition.variants,
        location,
        context,
      );
    default:
      throw new Error(`Unknown type definition type: ${typeDefinition.type}`);
  }
}

function createDeriveStructEqualsMethodNodes(identifier, fields, location, context) {
  return [
    ...getTemplateElements(
      context.import(TEMPLATE_EQUALS_STRUCT, {
        $struct_name: identifier,
        $field_names: createListDirective({
          elements: fields.map((field) => field.name),
          location,
        }),
      }).module,
    ),
    NodeType.Whitespace({ source: '\n', location }),
    ...fields.flatMap((field) => [
      NodeType.Whitespace({ source: '\n', location }),
      ...createDeriveStructFieldEqualsMethodNodes(
        identifier,
        field.name,
        field.type,
        location,
        context,
      ),
    ]),
  ];
}

function createDeriveStructFieldEqualsMethodNodes(
  identifier,
  fieldIdentifier,
  fieldType,
  location,
  context,
) {
  switch (fieldType.type) {
    case 'primitive':
      return getTemplateElements(
        context.import(TEMPLATE_EQUALS_FIELD_PRIMITIVE, {
          $type_name: identifier,
          $field_name: fieldIdentifier,
          $field_type: fieldType.target,
        }).module,
      );
    case 'reference':
      return getTemplateElements(
        context.import(
          fieldType.optional
            ? TEMPLATE_EQUALS_FIELD_OPTIONAL_REFERENCE
            : TEMPLATE_EQUALS_FIELD_REFERENCE,
          {
            $type_name: identifier,
            $field_name: fieldIdentifier,
            $target_type: fieldType.target,
          },
        ).module,
      );
    case 'struct':
    case 'union':
      return getTemplateElements(
        context.import(TEMPLATE_EQUALS_FIELD_INLINE, {
          $type_name: identifier,
          $field_name: fieldIdentifier,
          $target_type: fieldType.identifier,
        }).module,
      );
    case 'repeated':
      return [
        ...getTemplateElements(
          context.import(TEMPLATE_EQUALS_FIELD_REPEATED, {
            $type_name: identifier,
            $field_name: fieldIdentifier,
            $field_size: NodeType.Term({ source: getStructFieldTypeItemSize(fieldType), location }),
          }).module,
        ),
        ...createDeriveStructFieldEqualsMethodNodes(
          identifier,
          NodeType.Term({
            source: `${fieldIdentifier.source.slice('$'.length)}::item`,
            location,
          }),
          fieldType.target,
          location,
          context,
        ),
      ];
    default:
      throw new Error(`Unknown field type: ${fieldType.type}`);
  }
}

function createDeriveUnionEqualsMethodNodes(identifier, variants, location, context) {
  return getTemplateElements(
    context.import(TEMPLATE_EQUALS_UNION, {
      $union_name: identifier,
      $union_variants: createListDirective({
        elements: variants.map((variant) => variant.identifier),
        location,
      }),
    }).module,
  );
}

function createDeriveHashMethodNodes(typeDefinition, location, context) {
  switch (typeDefinition.type) {
    case 'struct':
      return createDeriveStructHashMethodNodes(
        typeDefinition.identifier,
        typeDefinition.fields,
        location,
        context,
      );
    case 'union':
      return createDeriveUnionHashMethodNodes(
        typeDefinition.identifier,
        typeDefinition.variants,
        location,
        context,
      );
    default:
      throw new Error(`Unknown type definition type: ${typeDefinition.type}`);
  }
}

function createDeriveStructHashMethodNodes(identifier, fields, location, context) {
  return [
    ...getTemplateElements(
      context.import(TEMPLATE_HASH_STRUCT, {
        $struct_name: identifier,
        $field_names: createListDirective({
          elements: fields.map((field) => field.name),
          location,
        }),
      }).module,
    ),
    NodeType.Whitespace({ source: '\n', location }),
    ...fields.flatMap((field) => [
      NodeType.Whitespace({ source: '\n', location }),
      ...createDeriveStructFieldHashMethodNodes(
        identifier,
        field.name,
        field.type,
        location,
        context,
      ),
    ]),
  ];
}

function createDeriveStructFieldHashMethodNodes(
  identifier,
  fieldIdentifier,
  fieldType,
  location,
  context,
) {
  switch (fieldType.type) {
    case 'primitive':
      return getTemplateElements(
        context.import(TEMPLATE_HASH_FIELD_PRIMITIVE, {
          $type_name: identifier,
          $field_name: fieldIdentifier,
          $field_type: fieldType.target,
        }).module,
      );
    case 'reference':
      return getTemplateElements(
        context.import(
          fieldType.optional
            ? TEMPLATE_HASH_FIELD_OPTIONAL_REFERENCE
            : TEMPLATE_HASH_FIELD_REFERENCE,
          {
            $type_name: identifier,
            $field_name: fieldIdentifier,
            $target_type: fieldType.target,
          },
        ).module,
      );
    case 'struct':
    case 'union':
      return getTemplateElements(
        context.import(TEMPLATE_HASH_FIELD_INLINE, {
          $type_name: identifier,
          $field_name: fieldIdentifier,
          $target_type: fieldType.identifier,
        }).module,
      );
    case 'repeated':
      return [
        ...getTemplateElements(
          context.import(TEMPLATE_HASH_FIELD_REPEATED, {
            $type_name: identifier,
            $field_name: fieldIdentifier,
            $field_size: NodeType.Term({ source: getStructFieldTypeItemSize(fieldType), location }),
          }).module,
        ),
        ...createDeriveStructFieldHashMethodNodes(
          identifier,
          NodeType.Term({
            source: `${fieldIdentifier.source.slice('$'.length)}::item`,
            location,
          }),
          fieldType.target,
          location,
          context,
        ),
      ];
    default:
      throw new Error(`Unknown field type: ${fieldType.type}`);
  }
}

function createDeriveUnionHashMethodNodes(identifier, variants, location, context) {
  return getTemplateElements(
    context.import(TEMPLATE_HASH_UNION, {
      $union_name: identifier,
      $union_variants: createListDirective({
        elements: variants.map((variant) => variant.identifier),
        location,
      }),
    }).module,
  );
}

function parseDeriveMethodName(node) {
  if (!isIdentifierNode(node)) return null;
  switch (node.source) {
    case '$size':
      return 'size';
    case '$equals':
      return 'equals';
    case '$hash':
      return 'hash';
    default:
      return null;
  }
}

function parseTypeDefinition(node, context) {
  if (isNamedInstructionNode(STRUCT_DIRECTIVE, node)) {
    const { identifier, fields } = parseStructDefinition(node, context);
    return {
      type: 'struct',
      identifier,
      fields,
      location: node.location,
    };
  } else if (isNamedInstructionNode(UNION_DIRECTIVE, node)) {
    const { identifier, variants } = parseUnionDefinition(node, context);
    return {
      type: 'union',
      identifier,
      variants,
      location: node.location,
    };
  } else {
    return null;
  }
}

function getTemplateElements(template) {
  return template.statements;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
