// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::IndexedAccessorIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createIntegersIterator,
      createIndexedAccessorIterator,
      createPair,
      createRepeatIterator,
      createString,
      createTakeIterator,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createIndexedAccessorIterator(createEmptyIterator([]), 0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createIndexedAccessorIterator(createEmptyIterator([]), 1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIndexedAccessorIterator(
              createTriple(
                createPair(createString('foo'), createInt(3)),
                createPair(createString('bar'), createInt(4)),
                createPair(createString('baz'), createInt(5)),
              ),
              0,
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["foo", "bar", "baz"]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIndexedAccessorIterator(
              createTriple(
                createPair(createString('foo'), createInt(3)),
                createPair(createString('bar'), createInt(4)),
                createPair(createString('baz'), createInt(5)),
              ),
              1,
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIndexedAccessorIterator(
              createTakeIterator(createRepeatIterator(createIntegersIterator()), 5),
              3,
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 3, 3, 3, 3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
