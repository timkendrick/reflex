// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::FlattenIterator', (test) => {
    test('iteration', (assert, {
      createEmptyIterator,
      createBuiltin,
      createInt,
      createFlattenIterator,
      createApplication,
      createPair,
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
          createUnitList(createFlattenIterator(createEmptyIterator())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFlattenIterator(
              createTriple(createEmptyIterator(), createEmptyIterator(), createEmptyIterator()),
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
            createFlattenIterator(
              createTriple(
                createUnitList(createInt(3)),
                createUnitList(createInt(4)),
                createUnitList(createInt(5)),
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
            createFlattenIterator(
              createTriple(
                createUnitList(createInt(3)),
                createPair(createInt(4), createInt(5)),
                createTriple(createInt(6), createInt(7), createInt(8)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFlattenIterator(
              createTriple(
                createTriple(createInt(3), createInt(4), createInt(5)),
                createPair(createInt(6), createInt(7)),
                createUnitList(createInt(8)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFlattenIterator(
              createTriple(
                createTriple(createInt(3), createInt(4), createInt(5)),
                createTriple(createInt(6), createInt(7), createInt(8)),
                createTriple(createInt(9), createInt(10), createInt(11)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8, 9, 10, 11]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
      createFlattenIterator,
      createMapIterator,
      createLambda,
      createRangeIterator,
      createSignal,
      createString,
      createTriple,
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
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFlattenIterator(
              createMapIterator(
                createTriple(
                  createSignal(createErrorCondition(createString('foo'))),
                  createRangeIterator(3, 3),
                  createSignal(createErrorCondition(createString('bar'))),
                ),
                createBuiltin(Stdlib.Identity),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFlattenIterator(
              createMapIterator(
                createRangeIterator(3, 3),
                createLambda(
                  1,
                  createApplication(createBuiltin(Stdlib.Throw), createUnitList(createVariable(0))),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:3>,<ErrorCondition:4>,<ErrorCondition:5>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
