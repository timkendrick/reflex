// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_StringifyJson', (test) => {
    test('(Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify(false)));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify(true)));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.StringifyJson),
        createUnitList(createNil()),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), JSON.stringify(JSON.stringify(null)));
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('1'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(-1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-1'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('3'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(-3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-3'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('123'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(-123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-123'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('2147483647'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createInt(-0x7fffffff)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-2147483647'));
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(0.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('0.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('1.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-1.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(3.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('3.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-3.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-3.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('3.142'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-3.142'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(2.718)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('2.718'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-2.718)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-2.718'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(123.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('123.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-123.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-123.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(123.45)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('123.45'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-123.45)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-123.45'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(2147483647.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('2147483647.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-2147483647.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify('-2147483647.0'));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify(null)));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(-Infinity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify(null)));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createFloat(NaN)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify(null)));
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('foo')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\b')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\b')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\f')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\f')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\n')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\n')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\r')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\r')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\t')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\t')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('"')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('\\')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('\\')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('""')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('""')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('"foo"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('"foo"')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createString('foo "bar" baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify('foo "bar" baz')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify([])));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createUnitList(createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify([3])));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createPair(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify([3, 4])));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify([3, 4, 5])));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createTriple(createInt(3), createBuiltin(Stdlib.Identity), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:StringifyJson([3, Identity, 5])>}',
        );
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify({})));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(
            createRecord(createUnitList(createString('foo')), createUnitList(createInt(3))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify({ foo: 3 })));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          JSON.stringify(JSON.stringify({ foo: 3, bar: 4, baz: 5 })),
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createInt(3), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(JSON.stringify({ foo: 3, baz: 5 })));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.StringifyJson),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createBuiltin(Stdlib.Identity), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:StringifyJson({ "foo": 3, "bar": Identity, "baz": 5 })>}',
        );
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
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.StringifyJson),
        createUnitList(createDate(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        JSON.stringify(JSON.stringify(new Date(timestamp).toISOString())),
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};
