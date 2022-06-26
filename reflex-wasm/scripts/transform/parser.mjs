// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { TokenType } from './lexer.mjs';
import { formatSourceRange, ParseError } from './utils.mjs';

export const NodeType = {
  Program({ statements, location }) {
    return {
      type: NodeType.Program,
      statements,
      location,
      modified: true,
    };
  },
  Instruction({ instruction, elements, location }) {
    return {
      type: NodeType.Instruction,
      instruction,
      elements,
      location,
      modified: true,
    };
  },
  Term({ source, location }) {
    return {
      type: NodeType.Term,
      source,
      location,
      modified: true,
    };
  },
  String({ source, location }) {
    return {
      type: NodeType.String,
      source,
      location,
      modified: true,
    };
  },
  Comment({ source, location }) {
    return {
      type: NodeType.Comment,
      source,
      location,
      modified: true,
    };
  },
  Whitespace({ source, location }) {
    return {
      type: NodeType.Whitespace,
      source,
      location,
      modified: true,
    };
  },
};

export function parse(tokens, source, path) {
  const state = {
    tokens,
    offset: 0,
    scope: {
      offset: 0,
      elements: [],
    },
    parentScopes: [],
    path,
    source,
  };
  while (state.offset < state.tokens.length) {
    const token = state.tokens[state.offset++];
    switch (token.type) {
      case TokenType.OpenParen: {
        state.parentScopes.push(state.scope);
        state.scope = {
          offset: token.location.offset,
          elements: [],
        };
        break;
      }
      case TokenType.CloseParen: {
        const parentScope = state.parentScopes.pop();
        if (!parentScope) {
          throw new ParseError(token.location, state.source, `Unexpected closing parenthesis`);
        }
        const childScope = state.scope;
        const childTerms = childScope.elements.filter((node) => {
          switch (node.type) {
            case NodeType.Whitespace:
            case NodeType.Comment:
              return false;
            default:
              return true;
          }
        });
        if (childTerms.length === 0) {
          throw new ParseError(
            token.location,
            state.source,
            `Empty instruction: ${formatSourceRange(state.source, token.location)}`,
          );
        }
        const location = {
          path: state.path,
          offset: childScope.offset,
          length: token.location.offset + token.location.length - childScope.offset,
        };
        const firstChild = childTerms[0];
        if (firstChild.type !== NodeType.Term) {
          throw new ParseError(
            location,
            state.source,
            `Invalid instruction: ${formatSourceRange(state.source, location)}`,
          );
        }
        state.scope = parentScope;
        state.scope.elements.push({
          type: NodeType.Instruction,
          instruction: firstChild.source,
          elements: childScope.elements,
          location,
          modified: false,
        });
        break;
      }
      case TokenType.String: {
        state.scope.elements.push({
          type: NodeType.String,
          source: state.source.slice(
            token.location.offset,
            token.location.offset + token.location.length,
          ),
          location: token.location,
          modified: false,
        });
        break;
      }
      case TokenType.Term: {
        state.scope.elements.push({
          type: NodeType.Term,
          source: state.source.slice(
            token.location.offset,
            token.location.offset + token.location.length,
          ),
          location: token.location,
          modified: false,
        });
        break;
      }
      case TokenType.Whitespace: {
        state.scope.elements.push({
          type: NodeType.Whitespace,
          source: state.source.slice(
            token.location.offset,
            token.location.offset + token.location.length,
          ),
          location: token.location,
          modified: false,
        });
        break;
      }
      case TokenType.LineComment: {
        state.scope.elements.push({
          type: NodeType.Comment,
          source: state.source.slice(
            token.location.offset,
            token.location.offset + token.location.length,
          ),
          location: token.location,
          modified: false,
        });
        break;
      }
      default: {
        throw new ParseError(
          token.location,
          state.source,
          `Unrecognized token: ${formatSourceRange(state.source, token.location)}`,
        );
      }
    }
  }
  if (state.parentScopes.length > 0) {
    const parentScope = state.parentScopes[state.parentScopes.length - 1];
    throw new ParseError(
      {
        path: state.path,
        offset: parentScope.offset,
        length: state.source.length - parentScope.offset,
      },
      state.source,
      'Unterminated instruction',
    );
  }
  if (!state.scope.elements.some((node) => node.type == NodeType.Instruction)) {
    throw new ParseError(
      { path: state.path, offset: 0, length: 0 },
      state.source,
      'Empty source file',
    );
  }
  return {
    type: NodeType.Program,
    statements: state.scope.elements,
    location: {
      path: state.path,
      offset: 0,
      length: state.source.length,
    },
    modified: false,
  };
}
