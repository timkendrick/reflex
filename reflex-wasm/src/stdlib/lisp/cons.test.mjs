// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Cons', (test) => {
    test('(Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cons),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Nil)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createNil,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cons),
          createPair(createInt(3), createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . null)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Tree)', (assert, {
      createApplication,
      createBuiltin,
      createTree,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cons),
          createPair(createInt(3), createTree(createInt(4), NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . (4 . NULL))');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cons),
          createPair(createInt(3), createTree(createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . (4 . 5))');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
