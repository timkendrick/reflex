// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ConstructHashset', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.ConstructHashset),
        createEmptyList(),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'Set(0)');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createString,
      createLambda,
      createUnitList,
      evaluate,
      hasHashsetValue,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(1)');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createUnitList(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(1)');
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
          ),
          false,
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createLambda,
      createPair,
      createString,
      evaluate,
      hasHashsetValue,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createPair(createString('foo'), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(2)');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createPair(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(2)');
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
          false,
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
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
      hasHashsetValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createTriple(createString('foo'), createString('bar'), createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(3)');
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashset),
          createTriple(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(3)');
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createApplication(createLambda(0, createString('qux')), createEmptyList()),
          ),
          false,
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
