// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::FilterIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createBoolean,
      createEmptyIterator,
      createBuiltin,
      createList,
      createNil,
      createFilterIterator,
      createRangeIterator,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createFilterIterator(createEmptyIterator(), createBuiltin(Stdlib.Not))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFilterIterator(createRangeIterator(3, 3), createBuiltin(Stdlib.Not)),
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
            createFilterIterator(
              createList([createNil(), createBoolean(false), createBoolean(true), createNil()]),
              createBuiltin(Stdlib.Not),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[null, false, null]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
      createFilterIterator,
      createInt,
      createLambda,
      createRangeIterator,
      createSignal,
      createString,
      createTriple,
      createUnitList,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createFilterIterator(
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
            createFilterIterator(
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
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
