// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

import { Rule } from 'eslint';
import * as ESTree from 'estree';

import { isExpression } from '../utils/ast';

const rule: Rule.RuleModule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce valid ReflexJS syntax',
    },
    schema: [],
  },
  create(context) {
    return {
      Program,
      ImportDeclaration,
      ExportNamedDeclaration: unimplemented,
      ExportAllDeclaration: unimplemented,
      ExportDefaultDeclaration,
      VariableDeclaration,
      VariableDeclarator,
      ReturnStatement,
      ThrowStatement,
      IfStatement,
      TryStatement,
      EmptyStatement,
      ExpressionStatement,
      SwitchStatement: unimplemented,
      FunctionDeclaration: unimplemented,
      ClassDeclaration: unimplemented,
      DebuggerStatement: unimplemented,
      WithStatement: unimplemented,
      LabeledStatement: unimplemented,
      BreakStatement: unimplemented,
      ContinueStatement: unimplemented,
      WhileStatement: unimplemented,
      DoWhileStatement: unimplemented,
      ForStatement: unimplemented,
      ForInStatement: unimplemented,
      ForOfStatement: unimplemented,
    };

    function ok(_node: ESTree.Node): void {
      return;
    }

    function err(
      node: ESTree.Node,
      message: string,
      data?: Record<string, string>,
    ): void {
      context.report({
        node,
        message,
        data,
      });
    }

    function unimplemented(node: ESTree.Node): void {
      return err(node, `Unsupported syntax: {{ node }}`, { node: node.type });
    }

    function Program(node: ESTree.Program) {
      const isScript = node.sourceType === 'script';
      const illegalStatements = node.body.filter((node) => {
        switch (node.type) {
          case 'ImportDeclaration':
          case 'VariableDeclaration':
          case 'ExportAllDeclaration':
          case 'ExportDefaultDeclaration':
          case 'ExportNamedDeclaration':
            return false;
          case 'ExpressionStatement':
            return !isScript;
          default:
            return true;
        }
      });
      for (const node of illegalStatements) {
        err(node, 'Unexpected top-level statement');
      }
      if (isScript) {
        const statements = node.body.filter((node) => {
          switch (node.type) {
            case 'ExpressionStatement':
              return true;
            default:
              return false;
          }
        });
        const [statement, ...trailing] = statements;
        if (!statement) err(node, 'Missing top-level expression');
        if (trailing.length > 0) err(node, 'Script contains unused statements');
      } else {
        const hasDefaultExport = node.body.some((node) => {
          switch (node.type) {
            case 'ExportDefaultDeclaration':
              return true;
            default:
              return false;
          }
        });
        if (!hasDefaultExport) {
          err(node, 'Missing default module export');
        }
      }
      return ok(node);
    }

    function ExpressionStatement(node: ESTree.ExpressionStatement): void {
      Expression(node.expression);
      return ok(node);
    }

    function ImportDeclaration(node: ESTree.ImportDeclaration): void {
      return ok(node);
    }

    function ExportDefaultDeclaration(
      node: ESTree.ExportDefaultDeclaration,
    ): void {
      if (!isExpression(node.declaration)) return unimplemented(node);
      return ok(node);
    }

    function VariableDeclaration(node: ESTree.VariableDeclaration): void {
      switch (node.kind) {
        case 'const':
          return ok(node);
        case 'let':
        case 'var':
        default:
          return unimplemented(node);
      }
    }

    function VariableDeclarator(node: ESTree.VariableDeclarator): void {
      VariableDeclaratorId(node.id);
      if (!node.init) return err(node, 'Missing variable initializer');
      return Expression(node.init);

      function VariableDeclaratorId(node: ESTree.Pattern): void {
        switch (node.type) {
          case 'Identifier':
            return ok(node);
          case 'ObjectPattern':
            return DestructuringObjectPattern(node);
          case 'ArrayPattern':
            return DestructuringArrayPattern(node);
          case 'RestElement':
          case 'AssignmentPattern':
          case 'MemberExpression':
          default:
            return unimplemented(node);
        }
      }
    }

    function DestructuringObjectPattern(node: ESTree.ObjectPattern): void {
      for (const property of node.properties) {
        DestructuringObjectPatternProperty(property);
      }
      return ok(node);

      function DestructuringObjectPatternProperty(
        node: ESTree.AssignmentProperty | ESTree.RestElement,
      ): void {
        switch (node.type) {
          case 'Property':
            if (node.computed) {
              if (node.key.type === 'PrivateIdentifier') {
                // https://github.com/estree/estree/blob/master/es2022.md#propertydefinition
                // > When key is a PrivateIdentifier, computed must be false.
                unimplemented(node);
              } else {
                DestructuringObjectPatternComputedPropertyKey(node.key);
              }
            } else {
              DestructuringObjectPatternStaticPropertyKey(node.key);
            }
            DestructuringObjectPatternPropertyValue(node.value);
            return ok(node);
          case 'RestElement':
          default:
            return unimplemented(node);
        }
      }

      function DestructuringObjectPatternStaticPropertyKey(
        node: ESTree.Expression | ESTree.PrivateIdentifier,
      ): void {
        switch (node.type) {
          case 'PrivateIdentifier':
            return unimplemented(node);
          case 'Identifier':
            return ok(node);
          case 'Literal':
          default:
            return DestructuringObjectPatternComputedPropertyKey(node);
        }
      }

      function DestructuringObjectPatternComputedPropertyKey(
        node: ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'Literal':
            switch (typeof node.value) {
              case 'string':
              case 'number':
              case 'boolean':
              case 'symbol':
              case 'undefined':
                return ok(node);
              case 'object':
                return node.value === null ? ok(node) : unimplemented(node);
              case 'bigint':
              default:
                return unimplemented(node);
            }
          default:
            return unimplemented(node);
        }
      }

      function DestructuringObjectPatternPropertyValue(
        node: ESTree.Pattern,
      ): void {
        switch (node.type) {
          case 'Identifier':
            return ok(node);
          case 'ObjectPattern':
          case 'ArrayPattern':
          case 'RestElement':
          case 'AssignmentPattern':
          case 'MemberExpression':
          default:
            return unimplemented(node);
        }
      }
    }

    function DestructuringArrayPattern(node: ESTree.ArrayPattern): void {
      for (const element of node.elements) {
        if (!element) continue;
        DestructuringArrayPatternElement(element);
      }
      return ok(node);

      function DestructuringArrayPatternElement(node: ESTree.Pattern): void {
        switch (node.type) {
          case 'Identifier':
            return ok(node);
          case 'RestElement':
            return unimplemented(node);
          case 'ObjectPattern':
          case 'ArrayPattern':
          case 'AssignmentPattern':
          case 'MemberExpression':
          default:
            return unimplemented(node);
        }
      }
    }

    function Expression(node: ESTree.Expression): void {
      switch (node.type) {
        case 'Identifier':
          return IdentifierExpression(node);
        case 'Literal':
          return LiteralExpression(node);
        case 'TemplateLiteral':
          return TemplateLiteralExpression(node);
        case 'TaggedTemplateExpression':
          return TaggedTemplateExpression(node);
        case 'UnaryExpression':
          return UnaryExpression(node);
        case 'BinaryExpression':
          return BinaryExpression(node);
        case 'LogicalExpression':
          return LogicalExpression(node);
        case 'ConditionalExpression':
          return ConditionalExpression(node);
        case 'ArrowFunctionExpression':
          return ArrowFunctionExpression(node);
        case 'MemberExpression':
          return MemberExpression(node);
        case 'CallExpression':
          return CallExpression(node);
        case 'NewExpression':
          return NewExpression(node);
        case 'ObjectExpression':
          return ObjectExpression(node);
        case 'ArrayExpression':
          return ArrayExpression(node);
        case 'ChainExpression':
          return unimplemented(node);
        case 'ClassExpression':
          return unimplemented(node);
        case 'FunctionExpression':
          return unimplemented(node);
        case 'AssignmentExpression':
        case 'AwaitExpression':
        case 'ImportExpression':
        case 'MetaProperty':
        case 'SequenceExpression':
        case 'ThisExpression':
        case 'UpdateExpression':
        case 'YieldExpression':
        default:
          return unimplemented(node);
      }
    }

    function IdentifierExpression(node: ESTree.Identifier): void {
      return ok(node);
    }

    function LiteralExpression(node: ESTree.Literal): void {
      switch (node.value) {
        case null:
        case undefined:
          return ok(node);
        default:
          switch (typeof node.value) {
            case 'string':
            case 'number':
            case 'boolean':
              return ok(node);
            default:
              return unimplemented(node);
          }
      }
    }

    function TemplateLiteralExpression(node: ESTree.TemplateLiteral): void {
      for (const expression of node.expressions) {
        Expression(expression);
      }
      return ok(node);
    }

    function TaggedTemplateExpression(
      node: ESTree.TaggedTemplateExpression,
    ): void {
      return TemplateLiteralExpression(node.quasi);
    }

    function UnaryExpression(node: ESTree.UnaryExpression): void {
      switch (node.operator) {
        case '-':
        case '+':
        case '!':
          return Expression(node.argument);
        case 'typeof':
          return unimplemented(node);
        case '~':
        case 'void':
        case 'delete':
        default:
          return unimplemented(node);
      }
    }

    function BinaryExpression(node: ESTree.BinaryExpression): void {
      switch (node.operator) {
        case '+':
        case '-':
        case '*':
        case '/':
        case '%':
        case '**':
        case '<':
        case '<=':
        case '>':
        case '>=':
        case '==':
        case '!=':
        case '===':
        case '!==':
        case 'in':
          Expression(node.left);
          Expression(node.right);
          return ok(node);
        case '<<':
        case '>>':
        case '>>>':
        case '|':
        case '^':
        case '&':
        case 'instanceof':
        default:
          return unimplemented(node);
      }
    }

    function LogicalExpression(node: ESTree.LogicalExpression): void {
      switch (node.operator) {
        case '||':
        case '&&':
          Expression(node.left);
          Expression(node.right);
          return ok(node);
        case '??':
          return unimplemented(node);
        default:
          return unimplemented(node);
      }
    }

    function ConditionalExpression(node: ESTree.ConditionalExpression): void {
      Expression(node.test);
      Expression(node.consequent);
      Expression(node.alternate);
      return ok(node);
    }

    function ArrowFunctionExpression(
      node: ESTree.ArrowFunctionExpression,
    ): void {
      if (node.async || node.generator) return unimplemented(node);
      for (const param of node.params) {
        FunctionParam(param);
      }
      return ArrowFunctionExpressionBody(node.body);

      function ArrowFunctionExpressionBody(
        node: ESTree.Expression | ESTree.BlockStatement,
      ): void {
        switch (node.type) {
          case 'BlockStatement':
            return assertValidBlock(node, node.body);
          default:
            return Expression(node);
        }
      }
    }

    function FunctionParam(node: ESTree.Pattern): void {
      switch (node.type) {
        case 'Identifier':
          return ok(node);
        case 'ObjectPattern':
          return DestructuringObjectPattern(node);
        case 'ArrayPattern':
          return DestructuringArrayPattern(node);
        case 'RestElement':
        case 'AssignmentPattern':
        case 'MemberExpression':
        default:
          return unimplemented(node);
      }
    }

    function MemberExpression(node: ESTree.MemberExpression): void {
      MemberExpressionObject(node.object);
      MemberExpressionProperty(node.property);
      return ok(node);

      function MemberExpressionObject(
        node: ESTree.Super | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'Super':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }

      function MemberExpressionProperty(
        node: ESTree.PrivateIdentifier | ESTree.Expression,
      ) {
        switch (node.type) {
          case 'PrivateIdentifier':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }
    }

    function CallExpression(node: ESTree.CallExpression): void {
      CallExpressionCallee(node.callee);
      for (const argument of node.arguments) {
        CallExpressionArgument(argument);
      }
      return ok(node);

      function CallExpressionCallee(
        node: ESTree.Super | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'Super':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }

      function CallExpressionArgument(
        node: ESTree.SpreadElement | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'SpreadElement':
            return Expression(node.argument);
          default:
            return Expression(node);
        }
      }
    }

    function NewExpression(node: ESTree.NewExpression): void {
      NewExpressionCallee(node.callee);
      for (const argument of node.arguments) {
        NewExpressionArgument(argument);
      }
      return ok(node);

      function NewExpressionCallee(
        node: ESTree.Super | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'Super':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }

      function NewExpressionArgument(
        node: ESTree.SpreadElement | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'SpreadElement':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }
    }

    function ObjectExpression(node: ESTree.ObjectExpression): void {
      for (const property of node.properties) {
        ObjectExpressionProperty(property);
      }
      return ok(node);

      function ObjectExpressionProperty(
        node: ESTree.Property | ESTree.SpreadElement,
      ): void {
        switch (node.type) {
          case 'SpreadElement':
            return Expression(node.argument);
          case 'Property':
            if (node.method) return unimplemented(node);
            if (node.kind === 'get') return unimplemented(node);
            if (node.kind === 'set') return unimplemented(node);
            if (node.computed) {
              if (node.key.type === 'PrivateIdentifier') {
                // https://github.com/estree/estree/blob/master/es2022.md#propertydefinition
                // > When key is a PrivateIdentifier, computed must be false.
                unimplemented(node);
              } else {
                ObjectExpressionComputedPropertyKey(node.key);
              }
            } else {
              ObjectExpressionStaticPropertyKey(node.key);
            }
            if (!node.shorthand) {
              ObjectExpressionPropertyValue(node.value);
            }
            return ok(node);
        }
      }

      function ObjectExpressionStaticPropertyKey(
        node: ESTree.Expression | ESTree.PrivateIdentifier,
      ): void {
        switch (node.type) {
          case 'PrivateIdentifier':
            return unimplemented(node);
          case 'Identifier':
            return ok(node);
          case 'Literal':
            switch (typeof node.value) {
              case 'string':
              case 'number':
                return ok(node);
              default:
                return unimplemented(node);
            }
          default:
            return ObjectExpressionComputedPropertyKey(node);
        }
      }

      function ObjectExpressionComputedPropertyKey(
        node: ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'Literal':
            switch (node.value) {
              case null:
              case undefined:
                return ok(node);
              default:
                switch (typeof node.value) {
                  case 'string':
                  case 'number':
                  case 'boolean':
                    return ok(node);
                  default:
                    return unimplemented(node);
                }
            }
          default:
            return unimplemented(node);
        }
      }

      function ObjectExpressionPropertyValue(
        node: ESTree.Expression | ESTree.Pattern,
      ): void {
        switch (node.type) {
          case 'ObjectPattern':
          case 'ArrayPattern':
          case 'RestElement':
          case 'AssignmentPattern':
            return unimplemented(node);
          default:
            return Expression(node);
        }
      }
    }

    function ArrayExpression(node: ESTree.ArrayExpression): void {
      for (const element of node.elements) {
        if (!element) {
          err(node, 'Missing array item');
          continue;
        }
        ArrayExpressionElement(element);
      }
      return ok(node);

      function ArrayExpressionElement(
        node: ESTree.SpreadElement | ESTree.Expression,
      ): void {
        switch (node.type) {
          case 'SpreadElement':
            return Expression(node.argument);
          default:
            return Expression(node);
        }
      }
    }

    function ReturnStatement(node: ESTree.ReturnStatement): void {
      if (!node.argument) return err(node, 'Missing return value');
      return Expression(node.argument);
    }

    function ThrowStatement(node: ESTree.ThrowStatement): void {
      return Expression(node.argument);
    }

    function IfStatement(node: ESTree.IfStatement): void {
      Expression(node.test);
      return ok(node);
    }

    function TryStatement(node: ESTree.TryStatement): void {
      if (node.finalizer) unimplemented(node.finalizer);
      if (!node.handler) return err(node, 'Missing catch block');
      if (node.handler.param) {
        TryStatementHandlerParam(node.handler.param);
      }
      return ok(node);

      function TryStatementHandlerParam(node: ESTree.Pattern): void {
        switch (node.type) {
          case 'Identifier':
            return ok(node);
          default:
            return unimplemented(node);
        }
      }
    }

    function EmptyStatement(node: ESTree.EmptyStatement): void {
      return ok(node);
    }

    function assertValidBlock(
      node: ESTree.Node,
      statements: Array<ESTree.Statement>,
    ): void {
      const blocks = parseBasicBlocks(statements, node);
      for (const { statements, parent } of blocks) {
        assertValidBasicBlock(statements, parent);
      }
      return ok(node);

      function parseBasicBlocks(
        statements: Array<ESTree.Statement>,
        parent: ESTree.Node,
      ): Array<{ statements: Array<ESTree.Statement>; parent: ESTree.Node }> {
        const blocks = new Array<{
          statements: Array<ESTree.Statement>;
          parent: ESTree.Node;
        }>();
        const currentBlock = {
          statements: new Array<ESTree.Statement>(),
          parent,
        };
        blocks.push(currentBlock);
        let node;
        const remaining = statements.slice();
        while ((node = remaining.shift())) {
          currentBlock.statements.push(node);
          switch (node.type) {
            case 'ReturnStatement':
            case 'ThrowStatement':
              break;
            case 'TryStatement':
              blocks.push(...parseBasicBlocks(node.block.body, node));
              if (node.handler) {
                blocks.push(
                  ...parseBasicBlocks(
                    node.handler.body.body,
                    node.handler.body,
                  ),
                );
              }
              if (node.finalizer) {
                blocks.push(
                  ...parseBasicBlocks(node.finalizer.body, node.finalizer),
                );
              }
              continue;
            case 'IfStatement': {
              const consequentBlock = parseIfStatementBranchBlock(
                node.consequent,
              );
              const alternateBlock = node.alternate
                ? parseIfStatementBranchBlock(node.alternate)
                : remaining.splice(0, remaining.length);
              blocks.push(...parseBasicBlocks(consequentBlock, node));
              blocks.push(...parseBasicBlocks(alternateBlock, node));
              continue;
            }
            default:
              continue;
          }
        }
        return blocks;
      }

      function parseIfStatementBranchBlock(
        node: ESTree.Statement,
      ): Array<ESTree.Statement> {
        switch (node.type) {
          case 'BlockStatement':
            return node.body;
          default:
            return [node];
        }
      }

      function assertValidBasicBlock(
        statements: Array<ESTree.Statement>,
        parent: ESTree.Node,
      ): void {
        const impureStatements = statements.filter((node) => {
          switch (node.type) {
            case 'ExpressionStatement':
              return true;
            default:
              return false;
          }
        });
        const nestedBlocks = statements.filter((node) => {
          switch (node.type) {
            case 'BlockStatement':
              return true;
            default:
              return false;
          }
        });
        const [tailStatement, ...trailing] = statements.filter((node) => {
          switch (node.type) {
            case 'ReturnStatement':
            case 'ThrowStatement':
            case 'IfStatement':
            case 'TryStatement':
              return true;
            case 'VariableDeclaration':
            case 'BlockStatement':
            case 'ExpressionStatement':
            case 'EmptyStatement':
            default:
              return false;
          }
        });
        for (const node of impureStatements) {
          err(node, 'Block contains unused statements');
        }
        for (const node of nestedBlocks) {
          err(node, 'Block contains nested blocks');
        }
        if (!tailStatement) {
          return err(parent, 'Block does not return a value');
        }
        if (trailing.length > 0) {
          return err(tailStatement, 'Block contains unused statements');
        }
        return ok(parent);
      }
    }
  },
};

export default rule;
