// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_If', (test) => {
    test('(Boolean, Lambda, Lambda)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createInt,
      createLambda,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(
            createBoolean(false),
            createLambda(0, createInt(3)),
            createLambda(0, createInt(4)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(
            createBoolean(true),
            createLambda(0, createInt(3)),
            createLambda(0, createInt(4)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Nil, Lambda, Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
      createNil,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(createNil(), createLambda(0, createInt(3)), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int, Lambda, Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(createInt(0), createLambda(0, createInt(3)), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(createInt(1), createLambda(0, createInt(3)), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(createInt(3), createLambda(0, createInt(3)), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(createInt(-1), createLambda(0, createInt(3)), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
