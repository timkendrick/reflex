// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ParseJson', (test) => {
    test('null', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.ParseJson),
        createUnitList(createString(JSON.stringify(null))),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'null');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('boolean', (assert, {
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
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify(false))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify(true))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('number', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      getFloatValue,
      getIntValue,
      isFloat,
      isInt,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), 0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result));
        assert.strictEqual(getIntValue(result), 1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), -1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), 3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), -3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString((0x7FFFFFFF).toString(10))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), 0x7FFFFFFF);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString((-0x7FFFFFFF).toString(10))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isInt(result))
        assert.strictEqual(getIntValue(result), -0x7FFFFFFF);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0e-3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0e-3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0E-3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 0E-3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e+0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e+0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E+0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E+0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e-0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e-0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E-0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E-0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1e+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 1E+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1e-3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 1e-3.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1E-3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 1E-3.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e+0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e+0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E+0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E+0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e-0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e-0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E-0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E-0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 3e-1.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 3E-1.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e4);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E4);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e+4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3e+4);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E+4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result), 3E+4);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3e-4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 3e-4.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3E-4')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result))
        assert.strictEqual(getFloatValue(result).toPrecision(10), 3E-4.toPrecision(10));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('0.0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 0.0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-1.0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), -1.0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3.0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 3.0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-3.0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), -3.0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-1.0123')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), -1.0123);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('3.142')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 3.142);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-3.142')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), -3.142);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('123.45')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 123.45);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('-123.45')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), -123.45);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E+0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E-0')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E-0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e+1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e+1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E-1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E-1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e+3')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e+3);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123e1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123e1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('1.0123E1')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isFloat(result));
        assert.strictEqual(getFloatValue(result), 1.0123E1);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('string', (assert, {
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
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify(''))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify('foo'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'foo');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify('"foo"'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify('foo "bar" baz'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'foo "bar" baz');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify('"\\/\b\f\n\r\t'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '"\\/\b\f\n\r\t');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('"\\u001f"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '\u001f');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('"\\u00d7"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '\u00d7');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('"\\u001F"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '\u001f');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('"\\u2705"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '\u2705');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString('"\\ud83c\\udf89"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '\ud83c\udf89');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('array', (assert, {
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
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify([]))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify(['foo', 'bar', 'baz']))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["foo", "bar", "baz"]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('object', (assert, {
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
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify({}))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ParseJson),
          createUnitList(createString(JSON.stringify({ foo: 'one', bar: 'two', baz: 'three' }))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": "one", "bar": "two", "baz": "three" }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
