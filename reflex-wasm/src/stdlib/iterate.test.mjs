// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Iterate', (test) => {
    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
      createRangeIterator,
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
          createUnitList(
            createApplication(createBuiltin(Stdlib.Iterate), createUnitList(createEmptyIterator())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createRangeIterator(3, 3)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
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
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Iterate), createUnitList(createEmptyList())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
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
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createRecord(createEmptyList(), createEmptyList())),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createRecord(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createTriple(createInt(6), createInt(7), createInt(8)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[3, 6], [4, 7], [5, 8]]');
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
      getListItems,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Iterate), createUnitList(createHashmap([]))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createHashmap([
                  [createInt(3), createInt(6)],
                  [createInt(4), createInt(7)],
                  [createInt(5), createInt(8)],
                ]),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        const items = getListItems(result).map(format);
        assert.strictEqual(items.length, 6);
        const entries = items
          .reduce((items, item, index) => {
            if (index % 2 === 0) {
              items.push([item, null]);
            } else {
              items[(index - 1) / 2][1] = item;
            }
            return items;
          }, [])
          .map(([key, value]) => `[${key}, ${value}]`);
        assert.ok(entries.includes('[3, 6]'));
        assert.ok(entries.includes('[4, 7]'));
        assert.ok(entries.includes('[5, 8]'));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Tree)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTree,
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
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(NULL, NULL)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(NULL, createInt(3))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), NULL)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), createInt(4))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
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
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(createInt(3), createTree(createInt(4), createTree(createInt(5), NULL))),
              ),
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
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(
                  createTree(createInt(3), NULL),
                  createTree(createInt(4), createTree(createInt(5), NULL)),
                ),
              ),
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
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(
                  createTree(createInt(3), createInt(4)),
                  createTree(createInt(5), createInt(6)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
