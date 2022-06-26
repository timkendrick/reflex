// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';

export default function transformDirectives(directives) {
  const transform = transformDirective(directives);
  return function transformDirectives(ast, context) {
    return transformRecursive(ast, context, transform) || [ast];
  };
}

function transformDirective(directives) {
  return (node, context) => {
    if (!isAnnotationNode(node)) return null;
    const directive = directives[node.instruction];
    if (!directive) return null;
    return directive(node, context);
  };
}

function transformRecursive(node, context, transform) {
  if (Array.isArray(node)) return transformMultiple(node, context, transform);
  if (isAnnotationNode(node)) return transform(node, context) || node;
  switch (node.type) {
    case NodeType.Program: {
      const transformedChildrenNode = transformProgramChildren(node, context, transform);
      const transformedNode = transform(transformedChildrenNode || node, context);
      return transformedNode || transformedChildrenNode;
    }
    case NodeType.Instruction: {
      const transformedChildrenNode = transformInstructionChildren(node, context, transform);
      const transformedNode = transform(transformedChildrenNode || node, context);
      return transformedNode || transformedChildrenNode;
    }
    case NodeType.Term:
    case NodeType.String:
    case NodeType.Comment:
    case NodeType.Whitespace:
    default: {
      return transform(node, context);
    }
  }
}

function transformProgramChildren(node, context, transform) {
  const transformedStatements = transformRecursive(node.statements, context, transform);
  if (!transformedStatements) return null;
  return [
    NodeType.Program({
      statements: transformedStatements,
      location: node.location,
    }),
  ];
}

function transformInstructionChildren(node, context, transform) {
  const transformedElements = transformRecursive(node.elements, context, transform);
  if (!transformedElements) return null;
  return [
    NodeType.Instruction({
      instruction: node.instruction,
      elements: transformedElements,
      location: node.location,
    }),
  ];
}

function transformMultiple(nodes, context, transform) {
  const transformedNodes = nodes.flatMap((node) => {
    const transformedNode = transformRecursive(node, context, transform);
    if (!transformedNode) return [node];
    if (Array.isArray(transformedNode)) return transformedNode;
    return [transformedNode];
  });
  if (
    transformedNodes.length === nodes.length &&
    transformedNodes.every((node, index) => node === nodes[index])
  ) {
    return null;
  }
  return transformedNodes;
}

function isAnnotationNode(node) {
  return node.type === NodeType.Instruction && node.instruction.startsWith('@');
}
