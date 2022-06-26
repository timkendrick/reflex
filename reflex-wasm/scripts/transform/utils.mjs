// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export class ParseError extends Error {
  constructor(location, source, message) {
    super(`${location ? formatSourceLocation(location, source) : '<unknown>'}: ${message}`);
  }
}

export function formatSourceLocation(location, source) {
  const { line, column } = parseSourceOffset(location.offset, source);
  return `${location.path}:${line}:${column}`;
}

export function formatSourceRange(source, location) {
  return JSON.stringify(source.slice(location.offset, location.offset + location.length));
}

function parseSourceOffset(offset, source) {
  let line = 1;
  let column = 1;
  let currentOffset = 0;
  while (currentOffset < source.length && currentOffset < offset) {
    switch (source.charAt(currentOffset++)) {
      case '\n': {
        line += 1;
        column = 1;
        break;
      }
      case '\r': {
        if (source.charAt(currentOffset) === '\n') {
          break;
        } else {
          line += 1;
          column = 1;
          break;
        }
      }
      default: {
        column += 1;
        break;
      }
    }
  }
  return { line, column };
}
