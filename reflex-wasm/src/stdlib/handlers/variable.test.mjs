// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Getter', (test) => {
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
          createBuiltin(Stdlib.Getter),
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
          createBuiltin(Stdlib.Getter),
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
  });

  describe('Stdlib_Setter', (test) => {
    test('(Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createSymbol,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Setter),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(2) => Effect("reflex::variable::set", [Symbol(123), Variable(1)], Variable(0))',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });

  describe('Stdlib_Variable', (test) => {
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
          createBuiltin(Stdlib.Variable),
          createPair(createSymbol(123), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[<!<CustomCondition:"reflex::variable::get":[Symbol(123), 3]:null>>, (2) => Effect("reflex::variable::set", [Symbol(123), Variable(1)], Variable(0))]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Variable),
              createPair(createSymbol(123), createInt(3)),
            ),
            createInt(0),
          ),
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
          createBuiltin(Stdlib.Get),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Variable),
              createPair(createSymbol(123), createInt(3)),
            ),
            createInt(0),
          ),
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
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Variable),
              createPair(createSymbol(123), createInt(3)),
            ),
            createInt(1),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(2) => Effect("reflex::variable::set", [Symbol(123), Variable(1)], Variable(0))',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
