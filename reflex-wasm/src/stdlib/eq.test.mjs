// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Eq', (test) => {
    test('(Nil, Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createNil(), createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Boolean, Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(false), createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(false), createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(true), createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(true), createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Boolean, Nil)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createNil,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(false), createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createBoolean(true), createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Nil, Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createNil,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createNil(), createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createNil(), createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(0), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(3), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(3), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Float, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(0), createFloat(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(2.718), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(3.142), createFloat(2.718)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(3.142), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(0), createFloat(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createInt(3), createFloat(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createFloat(3), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List, List)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(createEmptyList(), createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createTriple(createInt(3), createInt(4), createInt(6)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createTriple(createInt(3), createInt(4)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createTriple(createInt(3), createInt(4)),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('computed values', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createApplication(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
            createApplication(createBuiltin(Stdlib.Add), createPair(createInt(2), createInt(5))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Eq),
          createPair(
            createApplication(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
            createApplication(createBuiltin(Stdlib.Add), createPair(createInt(2), createInt(6))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
