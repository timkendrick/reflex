// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectList', (test) => {
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
      const expression = createApplication(createBuiltin(Stdlib.CollectList), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '[]');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(createApplication(createLambda(0, createInt(3)), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createPair(
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createTriple(
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
            createApplication(createLambda(0, createInt(5)), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
