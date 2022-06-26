// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const INCLUDE_DIRECTIVE = '@include';

export default function includeDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, includePath, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (
    !isNamedTermNode(INCLUDE_DIRECTIVE, instruction) ||
    !isStringNode(includePath) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${INCLUDE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const importedPath = ((value) => {
    try {
      const path = JSON.parse(value);
      if (typeof path !== 'string') throw null;
      return path;
    } catch (err) {
      throw new ParseError(node.location, context.sources.get(context.path), err.message);
    }
  })(includePath.source);
  const importedModule = ((path) => {
    try {
      return context.import(path, {});
    } catch (err) {
      throw new ParseError(node.location, context.sources.get(context.path), err.message);
    }
  })(importedPath);
  const rootModules = getModules(importedModule);
  if (rootModules.length === 0) {
    throw new ParseError(
      importedModule.location,
      context.sources.get(context.path),
      `Unable to include module ${importedPath}: Target file does not declare a top-level module`,
    );
  }
  if (rootModules.length > 1)
    throw new ParseError(
      importedModule.location,
      context.sources.get(context.path),
      `Unable to include module ${importedPath}: Target file declares multiple top-level modules`,
    );
  return [
    NodeType.Whitespace({
      source: '\n',
      location: node.location,
    }),
    NodeType.Comment({
      source: `;; ----- Begin ${INCLUDE_DIRECTIVE}: ${importedPath} -----\n`,
      location: node.location,
    }),
    ...getModuleInstructions(rootModules[0]),
    NodeType.Whitespace({
      source: '\n',
      location: node.location,
    }),
    NodeType.Comment({
      source: `;; ----- End ${INCLUDE_DIRECTIVE}: ${importedPath} -----\n`,
      location: node.location,
    }),
  ];
}

function getModules(ast) {
  return isProgramNode(ast) ? ast.statements.filter(isModuleNode) : [];
}

function getModuleInstructions(node) {
  const index = node.elements.findIndex((node) => isNamedTermNode('module', node));
  return trimWhitespaceNodes(
    index === -1
      ? node.elements
      : [...node.elements.slice(0, index), ...node.elements.slice(index + 1)],
  );
}

function trimWhitespaceNodes(nodes) {
  if (nodes.length === 0) return nodes;
  const startIndex = nodes.findIndex((node) => !isWhitespaceNode(node));
  const numTrailingWhitespaceNodes = nodes
    .slice()
    .reverse()
    .findIndex((node) => !isWhitespaceNode(node));
  const endIndex = nodes.length - numTrailingWhitespaceNodes;
  return nodes.slice(startIndex, endIndex);
}

function isModuleNode(node) {
  return isNamedInstructionNode('module', node);
}

function isProgramNode(node) {
  return node.type === NodeType.Program;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isStringNode(node) {
  return node.type === NodeType.String;
}

function isWhitespaceNode(node) {
  return node.type === NodeType.Whitespace;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}
