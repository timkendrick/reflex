// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Take', (test) => {
    test('(Iterator, Int)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
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
              createBuiltin(Stdlib.Take),
              createPair(createEmptyIterator(), createInt(0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createEmptyIterator(), createInt(3)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createInt(0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createInt(1)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createInt(2)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createInt(3)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createInt(4)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator, Float)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createFloat,
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
              createBuiltin(Stdlib.Take),
              createPair(createEmptyIterator(), createFloat(0.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createEmptyIterator(), createFloat(3.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createFloat(0.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createFloat(1.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createFloat(2.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createFloat(3.0)),
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
              createBuiltin(Stdlib.Take),
              createPair(createRangeIterator(3, 3), createFloat(4.0)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
