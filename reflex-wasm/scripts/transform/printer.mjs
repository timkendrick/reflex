// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from './parser.mjs';

export function print(ast, sources) {
  return (
    formatNode(ast, sources) ||
    (ast.location && !ast.modified
      ? formatSource(ast.location, sources)
      : ast.statements.map((statement) => formatSource(statement.location, sources)).join(''))
  );
}

function formatNode(node, sources) {
  switch (node.type) {
    case NodeType.Program:
      return formatProgram(node, sources);
    case NodeType.Instruction:
      return formatInstruction(node, sources);
    case NodeType.Term:
      return formatTerm(node, sources);
    case NodeType.String:
      return formatString(node, sources);
    case NodeType.Comment:
      return formatComment(node, sources);
    case NodeType.Whitespace:
      return formatWhitespace(node, sources);
    default:
      throw new Error(`Unexpected node type: ${node.type}`);
  }
}

function formatProgram(node, sources) {
  const formatted = formatNodes(node.statements, sources);
  return formatted && formatted.join('');
}

function formatInstruction(node, sources) {
  const formattedElements = formatNodes(ensureWhitespaceSeparators(node.elements), sources);
  return `(${(
    formattedElements || node.elements.map((element) => formatSource(element.location, sources))
  ).join('')})`;
}

function formatTerm(node, _sources) {
  return node.source;
}

function formatString(node, _sources) {
  return node.source;
}

function formatComment(node, _sources) {
  return node.source;
}

function formatWhitespace(node, _sources) {
  return node.source;
}

function formatNodes(nodes, sources) {
  return nodes.reduce((results, node, index, array) => {
    const formatted = node.location && !node.modified ? null : formatNode(node, sources);
    if (formatted) {
      results =
        results || array.slice(0, index).map((element) => formatSource(element.location, sources));
      results.push(formatted);
      return results;
    } else {
      if (results) results.push(formatSource(node.location, sources));
      return results;
    }
  }, null);
}

function ensureWhitespaceSeparators(nodes) {
  const hasAdjacentTerms = nodes.some(
    (node, index, array) =>
      index > 0 && node.type === NodeType.Term && array[index - 1].type === NodeType.Term,
  );
  if (hasAdjacentTerms) {
    return nodes.reduce((results, node, index, array) => {
      if (index > 0 && node.type === NodeType.Term && array[index - 1].type === NodeType.Term) {
        results.push(NodeType.Whitespace({ source: ' ', location: null }));
      }
      results.push(node);
      return results;
    }, []);
  } else {
    return nodes;
  }
}

function formatSource(location, sources) {
  const { path, offset, length } = location;
  const source = sources.get(path);
  if (!source) throw new Error(`Invalid source path: ${path}`);
  return source.slice(offset, offset + length);
}
