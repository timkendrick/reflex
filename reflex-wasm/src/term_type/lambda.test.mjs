// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Lambda', (test) => {
    test('format', (assert, { createLambda, createBoolean, format }) => {
      assert.strictEqual(format(createLambda(0, createBoolean(true))), '(0) => true');
      assert.strictEqual(format(createLambda(3, createBoolean(true))), '(3) => true');
    });

    test('hash', (assert, { createBoolean, createLambda, hash }) => {
      assert.strictEqual(
        hash(createLambda(0, createBoolean(true))),
        hash(createLambda(0, createBoolean(true))),
      );
      assert.strictEqual(
        hash(createLambda(3, createBoolean(true))),
        hash(createLambda(3, createBoolean(true))),
      );
      assert.notStrictEqual(
        hash(createLambda(0, createBoolean(true))),
        hash(createLambda(3, createBoolean(true))),
      );
      assert.notStrictEqual(
        hash(createLambda(0, createBoolean(true))),
        hash(createLambda(0, createBoolean(false))),
      );
    });

    test('equals', (assert, { createBoolean, createLambda, equals }) => {
      assert.strictEqual(
        equals(createLambda(0, createBoolean(true)), createLambda(0, createBoolean(true))),
        true,
      );
      assert.strictEqual(
        equals(createLambda(3, createBoolean(true)), createLambda(3, createBoolean(true))),
        true,
      );
      assert.strictEqual(
        equals(createLambda(0, createBoolean(true)), createLambda(3, createBoolean(true))),
        false,
      );
      assert.strictEqual(
        equals(createLambda(0, createBoolean(true)), createLambda(0, createBoolean(false))),
        false,
      );
    });

    test('arity', (assert, { createBoolean, createLambda, arity, FALSE }) => {
      assert.deepEqual(arity(createLambda(0, createBoolean(true))), [0, FALSE]);
      assert.deepEqual(arity(createLambda(1, createBoolean(true))), [1, FALSE]);
      assert.deepEqual(arity(createLambda(2, createBoolean(true))), [2, FALSE]);
    });

    test('basic lambda application', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createPair,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createLambda(0, createInt(3)), createEmptyList());
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(1, createVariable(0)),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(
            2,
            createApplication(
              createBuiltin(Stdlib.Add),
              createPair(createVariable(1), createVariable(0)),
            ),
          ),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(
            2,
            createApplication(
              createBuiltin(Stdlib.Subtract),
              createPair(createVariable(1), createVariable(0)),
            ),
          ),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('nested lambda applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
      createPair,
      createVariable,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createApplication(
            createLambda(
              1,
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.Subtract),
                  createPair(createVariable(1), createVariable(0)),
                ),
              ),
            ),
            createUnitList(createInt(3)),
          ),
          createUnitList(createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(
            2,
            createApplication(
              createApplication(
                createLambda(
                  1,
                  createLambda(
                    1,
                    createApplication(
                      createBuiltin(Stdlib.Subtract),
                      createPair(createVariable(1), createVariable(0)),
                    ),
                  ),
                ),
                createUnitList(createVariable(1)),
              ),
              createUnitList(createVariable(0)),
            ),
          ),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(
            2,
            createApplication(
              createApplication(
                createLambda(
                  2,
                  createLambda(
                    2,
                    createApplication(
                      createBuiltin(Stdlib.Subtract),
                      createPair(createVariable(1), createVariable(0)),
                    ),
                  ),
                ),
                createPair(createVariable(1), createVariable(0)),
              ),
              createPair(createVariable(1), createVariable(0)),
            ),
          ),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
