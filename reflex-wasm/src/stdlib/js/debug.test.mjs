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
      format,
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
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Boolean)', (assert, {
      createApplication,
      createBuiltin,
      createBoolean,
      createUnitList,
      evaluate,
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '1');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-1');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '2147483647');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createInt(-0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-2147483647');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createUnitList,
      evaluate,
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '1.0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-1.0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '3.142');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-3.142');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Infinity');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(-Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '-Infinity');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createFloat(NaN)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'NaN');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Date)', (assert, {
      createApplication,
      createBuiltin,
      createDate,
      createUnitList,
      evaluate,
      format,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.Debug),
        createUnitList(createDate(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isString(result));
      assert.strictEqual(getStringValue(result), `Date(${new Date(timestamp).toISOString()})`);
      assert.strictEqual(format(dependencies), 'NULL');
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
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createUnitList(createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createPair(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3, 4]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Debug),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
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
      format,
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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
