# eslint-plugin-reflex

ESLint plugin for ReflexJS

## Installation

You'll first need to install [ESLint](https://eslint.org/):

```sh
npm i eslint --save-dev
```

Next, install `eslint-plugin-reflex`:

```sh
npm install @marshallwace/eslint-plugin-reflex --save-dev
```

## Usage

Add `@marshallwace/reflex` to the plugins section of your `.eslintrc` configuration file. You can omit the `eslint-plugin-` prefix:

```json
{
    "plugins": [
        "@marshallwace/reflex"
    ]
}
```

To enable the recommended set of rules, add `plugin:@marshallwace/reflex/recommended` to the extends section:

```json
{
    "extends": [
        "plugin:@marshallwace/reflex/recommended"
    ]
}
```

Alternatively, configure the rules you want to use under the rules section.

```json
{
    "rules": {
        "@marshallwace/reflex/syntax": "error"
    }
}
```

## Rules

<!-- begin auto-generated rules list -->
TODO: Run eslint-doc-generator to generate the rules list.
<!-- end auto-generated rules list -->


