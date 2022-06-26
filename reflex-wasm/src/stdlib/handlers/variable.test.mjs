// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_GetVariable', (test) => {
    test('(Symbol, Int)', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createHashmap,
      createInt,
      createNil,
      createPair,
      createString,
      createSymbol,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.GetVariable),
          createPair(createSymbol(123), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<CustomCondition:"reflex::variable::get":[Symbol(123), 3]:null>}',
        );
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:"reflex::variable::get":[Symbol(123), 3]:null> . NULL)',
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.GetVariable),
          createPair(createSymbol(123), createInt(3)),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(
                createString('reflex::variable::get'),
                createPair(createSymbol(123), createInt(3)),
                createNil(),
              ),
              createInt(4),
            ],
            [
              createCustomCondition(
                createString('reflex::variable::get'),
                createPair(createSymbol(456), createInt(3)),
                createNil(),
              ),
              createInt(5),
            ],
          ]),
        );
        assert.strictEqual(format(result), '4');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:"reflex::variable::get":[Symbol(123), 3]:null> . NULL)',
        );
      })();
    });

    test('(Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.GetVariable),
        createPair(createInt(123), createInt(3)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<InvalidFunctionArgsCondition:GetVariable(123, 3)>}');
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });

  describe('Stdlib_SetVariable', (test) => {
    test('(Symbol, Int, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.SetVariable),
        createTriple(createSymbol(123), createInt(3), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:"reflex::variable::set":[Symbol(123), 3]:Symbol(456)>}',
      );
      assert.strictEqual(
        format(dependencies),
        '(<CustomCondition:"reflex::variable::set":[Symbol(123), 3]:Symbol(456)> . NULL)',
      );
    });

    test('(Int, Int, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.SetVariable),
        createTriple(createInt(123), createInt(3), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<InvalidFunctionArgsCondition:SetVariable(123, 3, Symbol(456))>}',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });

  describe('Stdlib_IncrementVariable', (test) => {
    test('(Symbol, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.IncrementVariable),
        createTriple(createSymbol(123), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:"reflex::variable::increment":[Symbol(123)]:Symbol(456)>}',
      );
      assert.strictEqual(
        format(dependencies),
        '(<CustomCondition:"reflex::variable::increment":[Symbol(123)]:Symbol(456)> . NULL)',
      );
    });

    test('(Int, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.IncrementVariable),
        createTriple(createInt(123), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<InvalidFunctionArgsCondition:IncrementVariable(123, Symbol(456))>}',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });

  describe('Stdlib_DecrementVariable', (test) => {
    test('(Symbol, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.DecrementVariable),
        createTriple(createSymbol(123), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:"reflex::variable::decrement":[Symbol(123)]:Symbol(456)>}',
      );
      assert.strictEqual(
        format(dependencies),
        '(<CustomCondition:"reflex::variable::decrement":[Symbol(123)]:Symbol(456)> . NULL)',
      );
    });

    test('(Int, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createSymbol,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.DecrementVariable),
        createTriple(createInt(123), createSymbol(456)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<InvalidFunctionArgsCondition:DecrementVariable(123, Symbol(456))>}',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};
