# Reflex tools

This package contains developer tools to assist in writing Reflex applications.

## Working with ReflexJS

ReflexJS only exposes a subset of ECMAScript syntax and APIs. This means that without any editor tools it can be difficult to know what constitutes a valid ReflexJS program.

The following tools are strongly recommended for developing ReflexJS applications:

- [`eslint-plugin-reflex`](./eslint-plugin-reflex): ESLint plugin that prevents usage of unsupported ECMAScript syntax
- [TypeScript types](./reflex-types): Type declarations for ReflexJS base library and builtin imports
