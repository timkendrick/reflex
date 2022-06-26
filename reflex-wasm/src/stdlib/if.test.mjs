// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_If', (test) => {
    test('(Boolean, Int, Int)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createInt,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createBoolean(false), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createBoolean(true), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Nil, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createNil,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createNil(), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createInt(0), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createInt(1), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createInt(3), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.If), createTriple(createInt(-1), createInt(3), createInt(4)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

  });
};
