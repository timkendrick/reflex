// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Slice', (test) => {
    test('(String, Int, Int)', (assert, {
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
          createBuiltin(Stdlib.Slice),
          createTriple(createString(''), createInt(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(0), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"a"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(0), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"ab"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(0), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"abc"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(0), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"abc"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(1), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(1), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"b"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(1), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"bc"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(1), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"bc"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(2), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(2), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"c"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(2), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"c"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(3), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(createString('abc'), createInt(3), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List, Int, Int)', (assert, {
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
          createBuiltin(Stdlib.Slice),
          createTriple(createEmptyList(), createInt(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(0),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(1),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(2),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(4),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(1),
            createInt(0),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(1),
            createInt(1),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(1),
            createInt(2),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(1),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(2),
            createInt(0),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(2),
            createInt(1),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(2),
            createInt(2),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(3),
            createInt(0),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Slice),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(3),
            createInt(1),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createRangeIterator,
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
              createBuiltin(Stdlib.Slice),
              createTriple(createEmptyIterator(), createInt(0), createInt(0)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(0), createInt(0)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(0), createInt(1)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(0), createInt(2)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(0), createInt(3)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(0), createInt(4)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(1), createInt(0)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(1), createInt(1)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(1), createInt(2)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(1), createInt(3)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(2), createInt(0)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(2), createInt(1)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(2), createInt(2)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(3), createInt(0)),
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
              createBuiltin(Stdlib.Slice),
              createTriple(createRangeIterator(3, 3), createInt(3), createInt(1)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
