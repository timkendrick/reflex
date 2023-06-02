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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
      createMapIterator,
      createLambda,
      createRangeIterator,
      createPair,
      createUnitList,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createBuiltin(Stdlib.ConstructList),
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
          '[{<ErrorCondition:3>}, {<ErrorCondition:4>}, {<ErrorCondition:5>}]',
        );
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
