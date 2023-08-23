// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Push', (test) => {
    test('(List, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Push),
          createPair(createEmptyList(), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Push),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashset, Int)', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createInt,
      createPair,
      hasHashsetValue,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Push),
          createPair(createHashset([]), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(1)');
        assert.strictEqual(hasHashsetValue(result, createInt(3)), true);
        assert.strictEqual(hasHashsetValue(result, createInt(4)), false);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Push),
          createPair(createHashset([createInt(3), createInt(4), createInt(5)]), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Set(4)');
        assert.strictEqual(hasHashsetValue(result, createInt(3)), true);
        assert.strictEqual(hasHashsetValue(result, createInt(4)), true);
        assert.strictEqual(hasHashsetValue(result, createInt(5)), true);
        assert.strictEqual(hasHashsetValue(result, createInt(6)), true);
        assert.strictEqual(hasHashsetValue(result, createInt(7)), false);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createPair,
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
            createApplication(
              createBuiltin(Stdlib.Push),
              createPair(createEmptyIterator(), createInt(3)),
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
              createBuiltin(Stdlib.Push),
              createPair(createRangeIterator(3, 3), createInt(6)),
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
