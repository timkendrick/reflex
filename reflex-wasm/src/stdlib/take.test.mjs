// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Take', (test) => {
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
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createInt(0),
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
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createInt(3),
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
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(0),
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
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(1),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(2),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(3),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Take),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(4),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
