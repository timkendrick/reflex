// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectHashset', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(createBuiltin(Stdlib.CollectHashset), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'Set(0)');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(String, String, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createLambda,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      hasHashsetValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashset),
          createTriple(createString('foo'), createString('bar'), createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(3)');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashset),
          createTriple(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(3)');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
