// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Filter', (test) => {
    test('(Iterator, Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createLambda,
      createPair,
      createRangeIterator,
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
            createApplication(
              createBuiltin(Stdlib.Filter),
              createPair(
                createEmptyIterator(),
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Eq),
                    createPair(
                      createApplication(
                        createBuiltin(Stdlib.Remainder),
                        createPair(createVariable(0), createInt(2)),
                      ),
                      createInt(0),
                    ),
                  ),
                ),
              ),
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
            createApplication(
              createBuiltin(Stdlib.Filter),
              createPair(
                createRangeIterator(3, 5),
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Eq),
                    createPair(
                      createApplication(
                        createBuiltin(Stdlib.Remainder),
                        createPair(createVariable(0), createInt(2)),
                      ),
                      createInt(0),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 6]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('signals', (assert, {
      createApplication,
      createBuiltin,
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
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Filter),
              createPair(
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
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
