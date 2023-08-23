// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Debug', (test) => {
    test('(Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Debug),
        createUnitList(createNil()),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isString(result));
      assert.strictEqual(getStringValue(result), 'null');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Boolean)', (assert, {
      createApplication,
      createBuiltin,
      createBoolean,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '1');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-1');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '2147483647');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-2147483647');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(0.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '0.0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '1.0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-1.0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '3.142');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-3.142');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Infinity');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-Infinity');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(NaN)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'NaN');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '""');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Timestamp)', (assert, {
      createApplication,
      createBuiltin,
      createTimestamp,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.Debug),
        createUnitList(createTimestamp(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isString(result));
      assert.strictEqual(getStringValue(result), `Timestamp(${new Date(timestamp).toISOString()})`);
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(List)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createPair,
      createTriple,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createUnitList(createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createPair(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '{}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(
            createRecord(createUnitList(createString('foo')), createUnitList(createInt(3))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '{ "foo": 3 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createInt(3), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '{ "foo": 3, 3: 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
