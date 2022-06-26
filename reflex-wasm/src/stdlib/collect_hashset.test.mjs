// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectHashset', (test) => {
    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      hasHashsetValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashset),
          createUnitList(createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set({0})');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashset),
          createUnitList(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set({3})');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
