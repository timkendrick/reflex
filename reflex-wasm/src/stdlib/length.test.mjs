// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Length', (test) => {
    test('(String)', (assert, {
      createApplication,
      createBuiltin,
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
          createBuiltin(Stdlib.Length),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
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
          createBuiltin(Stdlib.Length),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createRecord,
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
          createBuiltin(Stdlib.Length),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createRecord(
              createTriple(createInt(3), createInt(4), createInt(5)),
              createTriple(createInt(6), createInt(7), createInt(8)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createHashmap([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createHashmap([
              [createInt(3), createInt(6)],
              [createInt(4), createInt(7)],
              [createInt(5), createInt(8)],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashset)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashset,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createHashset([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createHashset([createInt(3), createInt(4), createInt(5)])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
      createFilterIterator,
      createInt,
      createLambda,
      createPair,
      createRangeIterator,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createRangeIterator(3, 3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createFilterIterator(
              createRangeIterator(3, 3),
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.Equal),
                  createPair(
                    createApplication(
                      createBuiltin(Stdlib.Remainder),
                      createPair(createVariable(0), createInt(2)),
                    ),
                    createInt(1),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '2');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Tree)', (assert, {
      createApplication,
      createTree,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTree(NULL, NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTree(NULL, createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '1');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTree(createInt(3), NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '1');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '2');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createTree(createInt(3), createTree(createInt(4), createTree(createInt(5), NULL))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createTree(
              createTree(createInt(3), NULL),
              createTree(createInt(4), createTree(createInt(5), NULL)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Length),
          createUnitList(
            createTree(
              createTree(createInt(3), createInt(4)),
              createTree(createInt(5), createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
