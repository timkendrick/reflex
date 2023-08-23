// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Fold', (test) => {
    test('(List, Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createEmptyList(), createInt(3), createBuiltin(Stdlib.Subtract)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(6),
            createBuiltin(Stdlib.Subtract),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test.skip('(Record, Int, Builtin)', (assert, {}) => {
      throw new Error('Not yet implemented');
    });

    test.skip('(Hashmap, Int, Builtin)', (assert, {}) => {
      throw new Error('Not yet implemented');
    });

    test('(Tree, Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createTree,
      createInt,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(
            createTree(createInt(3), createInt(4)),
            createInt(5),
            createBuiltin(Stdlib.Subtract),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${5 - 3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator, Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createRangeIterator,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createEmptyIterator(), createInt(3), createBuiltin(Stdlib.Subtract)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createRangeIterator(3, 3), createInt(6), createBuiltin(Stdlib.Subtract)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
