# Reflex Types

ReflexJS only exposes a subset of ECMAScript syntax and APIs. These type declarations override the standard ECMAScript base library types with the subset supported by ReflexJS, as well as exposing the builtin library modules.

## Getting started

Add the following configuration to your `tsconfig.json` to prevent loading the standard ECMAScript base library:

```json
{
  "compilerOptions": {
    "noLib": true
  }
}
```

Then create an `index.d.ts` file within your project that imports the type definitions:

```typescript
import '@marshallwace/reflex-types';
```
