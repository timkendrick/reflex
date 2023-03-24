// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Map', (test) => {
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
              createBuiltin(Stdlib.Map),
              createPair(
                createEmptyIterator(),
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Multiply),
                    createPair(createVariable(0), createInt(2)),
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
              createBuiltin(Stdlib.Map),
              createPair(
                createRangeIterator(3, 3),
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Multiply),
                    createPair(createVariable(0), createInt(2)),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[6, 8, 10]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
