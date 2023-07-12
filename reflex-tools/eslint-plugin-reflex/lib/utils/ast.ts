// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

import type * as ESTree from 'estree';

const EXPRESSION_NODES = {
  ArrayExpression: 'ArrayExpression',
  ArrowFunctionExpression: 'ArrowFunctionExpression',
  AssignmentExpression: 'AssignmentExpression',
  AwaitExpression: 'AwaitExpression',
  BinaryExpression: 'BinaryExpression',
  CallExpression: 'CallExpression',
  ChainExpression: 'ChainExpression',
  ClassExpression: 'ClassExpression',
  ConditionalExpression: 'ConditionalExpression',
  FunctionExpression: 'FunctionExpression',
  Identifier: 'Identifier',
  ImportExpression: 'ImportExpression',
  Literal: 'Literal',
  LogicalExpression: 'LogicalExpression',
  MemberExpression: 'MemberExpression',
  MetaProperty: 'MetaProperty',
  NewExpression: 'NewExpression',
  ObjectExpression: 'ObjectExpression',
  SequenceExpression: 'SequenceExpression',
  TaggedTemplateExpression: 'TaggedTemplateExpression',
  TemplateLiteral: 'TemplateLiteral',
  ThisExpression: 'ThisExpression',
  UnaryExpression: 'UnaryExpression',
  UpdateExpression: 'UpdateExpression',
  YieldExpression: 'YieldExpression',
};

export function isExpression(node: ESTree.Node): node is ESTree.Expression {
  return node.type in EXPRESSION_NODES;
}
