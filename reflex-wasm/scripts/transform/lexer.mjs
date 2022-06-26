// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { formatSourceRange, ParseError } from './utils.mjs';

export const TokenType = {
  OpenParen(char, _state) {
    return char === '(';
  },
  CloseParen(char, _state) {
    return char === ')';
  },
  String(char, _state) {
    return char == '"';
  },
  Term(char, state) {
    return (
      !TokenType.OpenParen(char, state) &&
      !TokenType.CloseParen(char, state) &&
      !TokenType.String(char, state) &&
      !TokenType.Whitespace(char, state) &&
      !TokenType.LineComment(char, state)
    );
  },
  Whitespace(char, _state) {
    switch (char) {
      case ' ':
      case '\t':
      case '\n':
      case '\r':
        return true;
      default:
        return false;
    }
  },
  LineComment(char, state) {
    return char === ';' && state.source.charAt(state.offset + 1) === ';';
  },
  // TODO: Support block comments: https://webassembly.github.io/spec/core/text/lexical.html#comments
};

export function tokenize(source, path) {
  const state = {
    tokens: [],
    offset: 0,
    source,
    path,
  };
  while (state.offset < state.source.length) {
    const char = state.source.charAt(state.offset);
    if (TokenType.OpenParen(char, state)) {
      state.tokens.push({
        type: TokenType.OpenParen,
        location: {
          path: state.path,
          offset: state.offset,
          length: 1,
        },
      });
      state.offset += 1;
    } else if (TokenType.CloseParen(char, state)) {
      state.tokens.push({
        type: TokenType.CloseParen,
        location: {
          path: state.path,
          offset: state.offset,
          length: 1,
        },
      });
      state.offset += 1;
    } else if (TokenType.Whitespace(char, state)) {
      const initialOffset = state.offset;
      state.offset += 1;
      while (
        state.offset < state.source.length &&
        TokenType.Whitespace(state.source.charAt(state.offset), state)
      ) {
        state.offset += 1;
      }
      state.tokens.push({
        type: TokenType.Whitespace,
        location: {
          path: state.path,
          offset: initialOffset,
          length: state.offset - initialOffset,
        },
      });
    } else if (TokenType.LineComment(char, state)) {
      const initialOffset = state.offset;
      state.offset += ';;'.length;
      while (state.offset < state.source.length && state.source.charAt(state.offset) !== '\n') {
        state.offset += 1;
      }
      if (state.source.charAt(state.offset) === '\n') {
        state.offset += 1;
      }
      state.tokens.push({
        type: TokenType.LineComment,
        location: {
          path: state.path,
          offset: initialOffset,
          length: state.offset - initialOffset,
        },
      });
    } else if (TokenType.String(char, state)) {
      const initialOffset = state.offset;
      state.offset += '"'.length;
      while (state.offset < state.source.length && state.source.charAt(state.offset) !== '"') {
        if (state.source.charAt(state.offset) === '\\') {
          state.offset += Math.min(state.source.length - state.offset, 2);
        } else {
          state.offset += 1;
        }
      }
      if (state.offset < state.source.length) {
        state.offset += '"'.length;
      } else {
        const location = {
          path: state.path,
          offset: initialOffset,
          length: state.offset - initialOffset,
        };
        throw new ParseError(
          location,
          state.source,
          `Unterminated string literal: ${formatSourceRange(state.source, location)}`,
        );
      }
      state.tokens.push({
        type: TokenType.String,
        location: {
          path: state.path,
          offset: initialOffset,
          length: state.offset - initialOffset,
        },
      });
    } else if (TokenType.Term(char, state)) {
      const initialOffset = state.offset;
      state.offset += 1;
      while (
        state.offset < state.source.length &&
        TokenType.Term(state.source.charAt(state.offset), state)
      ) {
        state.offset += 1;
      }
      state.tokens.push({
        type: TokenType.Term,
        location: {
          path: state.path,
          offset: initialOffset,
          length: state.offset - initialOffset,
        },
      });
    } else {
      const location = { path: state.path, offset: state.offset, length: 1 };
      throw new ParseError(
        location,
        state.source,
        `Invalid token: ${formatSourceRange(state.source, location)}`,
      );
    }
  }
  return state.tokens;
}
