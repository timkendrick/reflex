// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectConstructor', (test) => {
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
      const expression = createApplication(
        createBuiltin(Stdlib.CollectConstructor),
        createEmptyList(),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'Constructor({})');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(String, String, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createLambda,
      createTriple,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectConstructor),
          createTriple(createString('foo'), createString('bar'), createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Constructor({"foo", "bar", "baz"})');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectConstructor),
          createTriple(
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Constructor({"foo", "bar", "baz"})');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('Duplicate keys', (assert, {
      createApplication,
      createBuiltin,
      createTriple,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectConstructor),
          createTriple(createString('foo'), createString('bar'), createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Constructor({"foo", "bar"})');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
