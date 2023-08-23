// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectString', (test) => {
    test('()', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createEmptyList(),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createLambda,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createUnitList(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
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
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createPair(createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createPair(createString('foo'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createPair(createString(''), createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createPair(createString('foo'), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobar"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createPair(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobar"');
        assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString(''), createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString('foo'), createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString(''), createString('foo'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString(''), createString(''), createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString('foo'), createString('bar'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobar"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString('foo'), createString(''), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobar"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString(''), createString('foo'), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobar"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString('foo'), createString('bar'), createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobarbaz"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foobarbaz"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(String, Int, String)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectString),
          createTriple(createString('foo'), createInt(3), createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:CollectString(["foo", 3, "baz"])>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
