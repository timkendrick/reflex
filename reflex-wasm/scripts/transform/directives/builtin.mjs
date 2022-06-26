// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import path from 'path';
import url from 'url';

import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createListDirective } from './list.mjs';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const TEMPLATE = path.join(__dirname, '../templates/builtin.wat');

export const BUILTIN_DIRECTIVE = '@builtin';

export default function builtinDirective(node, context) {
  const [instruction, builtinIdentifier, signature, ...implementations] = node.elements
    .filter((node) => !isNonFunctionalNode(node))
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]));
  const argDefinitions = signature && parseArgDefinitions(signature);
  const builtinImplementations =
    argDefinitions && parseMethodImplementations(implementations, argDefinitions, context);
  const defaultImplementation =
    builtinImplementations && builtinImplementations.find(({ argTypes }) => !argTypes);
  if (
    !isNamedTermNode(BUILTIN_DIRECTIVE, instruction) ||
    !builtinIdentifier ||
    !isIdentifierNode(builtinIdentifier) ||
    !argDefinitions ||
    !builtinImplementations ||
    !defaultImplementation
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${BUILTIN_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    ...getTemplateElements(
      context.import(TEMPLATE, {
        $builtin_name: builtinIdentifier,
        $arg_names: createListDirective({
          elements: argDefinitions.map(({ identifier }) => identifier),
          location: node.location,
        }),
        $eager_arg_names: createListDirective({
          elements: argDefinitions
            .map(({ identifier, type }) =>
              type === 'strict' || type === 'eager' ? identifier : null,
            )
            .filter(Boolean),
          location: node.location,
        }),
        $strict_arg_names: createListDirective({
          elements: argDefinitions
            .map(({ identifier, type }) => (type === 'strict' ? identifier : null))
            .filter(Boolean),
          location: node.location,
        }),
        $implementation_names: createListDirective({
          elements: builtinImplementations
            .filter(({ argTypes }) => argTypes)
            .map(({ identifier }) => identifier),
          location: node.location,
        }),
        $implementation_signatures: createListDirective({
          elements: builtinImplementations
            .filter(({ argTypes }) => argTypes)
            .map(({ argTypes, location }) => createListDirective({ elements: argTypes, location })),
          location: node.location,
        }),
        $default_implementation: defaultImplementation.identifier,
      }),
    ),
    ...builtinImplementations.flatMap(({ implementation }) => [
      NodeType.Whitespace({ source: '\n\n', location: node.location }),
      implementation,
    ]),
  ];
}

function parseArgDefinitions(node) {
  if (!isNamedAnnotationNode('args', node)) return null;
  const [_instruction, ...varArgs] = node.elements.filter((arg) => !isNonFunctionalNode(arg));
  const argTypes = varArgs.map((arg) => parseArgSignatureNode(arg));
  if (!argTypes.every(Boolean)) return null;
  return argTypes;
}

function parseArgSignatureNode(node) {
  return (
    parseStrictArgSignatureNode(node) ||
    parseEagerArgSignatureNode(node) ||
    parseLazyArgSignatureNode(node)
  );
}

function parseStrictArgSignatureNode(node) {
  if (!isNamedAnnotationNode('strict', node)) return null;
  const [_instruction, argName, ...varArgs] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (!argName || !isTermNode(argName) || varArgs.length > 0) return null;
  return { identifier: argName, type: 'strict' };
}

function parseEagerArgSignatureNode(node) {
  if (!isNamedAnnotationNode('eager', node)) return null;
  const [_instruction, argName, ...varArgs] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (!argName || !isTermNode(argName) || varArgs.length > 0) return null;
  return { identifier: argName, type: 'eager' };
}

function parseLazyArgSignatureNode(node) {
  if (!isNamedAnnotationNode('lazy', node)) return null;
  const [_instruction, argName, ...varArgs] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (!argName || !isTermNode(argName) || varArgs.length > 0) return null;
  return { identifier: argName, type: 'lazy', location: node.location };
}

function parseMethodImplementations(nodes, signature, context) {
  const lastNode = nodes[nodes.length - 1];
  const defaultImplementation = lastNode && parseDefaultMethodImplementation(lastNode, context);
  if (!defaultImplementation) return null;
  const builtinOverloads = nodes
    .slice(0, -1)
    .map((node) => parseMethodImplementation(node, signature, context));
  if (!builtinOverloads.every(Boolean)) return null;
  return [...builtinOverloads, defaultImplementation];
}

function parseDefaultMethodImplementation(node, context) {
  if (!isNamedAnnotationNode('default', node)) return null;
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [_instruction, implementation, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const identifier = getNamedFunctionNodeIdentifier(implementation);
  if (!implementation || !identifier || varArgs.length > 0) return null;
  return { identifier, argTypes: null, implementation, location: node.location };
}

function parseMethodImplementation(node, signature, context) {
  if (!isNamedAnnotationNode('impl', node)) return null;
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [_instruction, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  const argTypes = varArgs.slice(0, -1).map((arg) => (isInstructionNode(arg) ? arg : null));
  const implementation = varArgs[varArgs.length - 1];
  const identifier = getNamedFunctionNodeIdentifier(implementation);
  if (
    argTypes.length !== signature.length ||
    !argTypes.every(Boolean) ||
    !implementation ||
    !identifier
  )
    return null;
  return { identifier, argTypes, implementation, location: node.location };
}

function getNamedFunctionNodeIdentifier(node) {
  if (!isNamedInstructionNode('func', node)) return null;
  const functionName = node.elements.filter((node) => !isNonFunctionalNode(node))[1];
  if (!isTermNode(functionName)) return null;
  return functionName;
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

function isNamedAnnotationNode(name, node) {
  return isNamedInstructionNode(`@${name}`, node);
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}

function isInstructionNode(node) {
  return node.type === NodeType.Instruction;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}
