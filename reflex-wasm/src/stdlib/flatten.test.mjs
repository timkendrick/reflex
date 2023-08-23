// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Flatten', (test) => {
    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
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
            createApplication(createBuiltin(Stdlib.Flatten), createUnitList(createEmptyIterator())),
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
              createBuiltin(Stdlib.Flatten),
              createUnitList(createUnitList(createEmptyIterator())),
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
              createBuiltin(Stdlib.Flatten),
              createUnitList(
                createTriple(createEmptyIterator(), createEmptyIterator(), createEmptyIterator()),
              ),
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
              createBuiltin(Stdlib.Flatten),
              createUnitList(
                createTriple(
                  createRangeIterator(3, 3),
                  createRangeIterator(6, 2),
                  createRangeIterator(8, 1),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
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
            createApplication(
              createBuiltin(Stdlib.Flatten),
              createUnitList(createSignal(createErrorCondition(createString('foo')))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Flatten),
              createUnitList(
                createTriple(
                  createSignal(createErrorCondition(createString('foo'))),
                  createRangeIterator(3, 3),
                  createSignal(createErrorCondition(createString('bar'))),
                ),
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
            createApplication(
              createBuiltin(Stdlib.Flatten),
              createUnitList(
                createMapIterator(
                  createRangeIterator(3, 3),
                  createLambda(
                    1,
                    createApplication(
                      createBuiltin(Stdlib.Throw),
                      createUnitList(createVariable(0)),
                    ),
                  ),
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
