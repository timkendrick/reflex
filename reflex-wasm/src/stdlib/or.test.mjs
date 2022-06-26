// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Or', (test) => {
    test('(Boolean, Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(false), createBoolean(false)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(false), createBoolean(true)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(true), createBoolean(false)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(true), createBoolean(true)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createNil(), createBoolean(false)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createNil(), createBoolean(true)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(false), createNil()));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(true), createNil()));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Boolean, Int)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(false), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Or), createPair(createBoolean(true), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
