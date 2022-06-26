// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Chain', (test) => {
    test('(Iterator, Iterator)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Chain),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
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
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Chain),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(1), createInt(2), createInt(3))),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[1, 2, 3]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Chain),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(1), createInt(2), createInt(3))),
                ),
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[1, 2, 3]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Chain),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(1), createInt(2), createInt(3))),
                ),
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(4), createInt(5), createInt(6))),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[1, 2, 3, 4, 5, 6]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
