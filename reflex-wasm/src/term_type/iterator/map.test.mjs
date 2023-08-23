// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::MapIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createEmptyIterator,
      createEvaluateIterator,
      createBuiltin,
      createInt,
      createMapIterator,
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
            createEvaluateIterator(
              createMapIterator(createEmptyIterator(), createBuiltin(Stdlib.Abs)),
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
            createEvaluateIterator(
              createMapIterator(
                createTriple(createInt(-3), createInt(-4), createInt(-5)),
                createBuiltin(Stdlib.Abs),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
      createInt,
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
            createMapIterator(
              createTriple(
                createSignal(createErrorCondition(createString('foo'))),
                createInt(3),
                createSignal(createErrorCondition(createString('bar'))),
              ),
              createBuiltin(Stdlib.Identity),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createMapIterator(
              createRangeIterator(3, 3),
              createLambda(
                1,
                createApplication(createBuiltin(Stdlib.Throw), createUnitList(createVariable(0))),
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
