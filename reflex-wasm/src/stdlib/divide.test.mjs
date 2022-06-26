// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Divide', (test) => {
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
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(0), createInt(0)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Divide(0, 0)> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(3), createInt(0)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Divide(3, 0)> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(0), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${0 / 3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(3), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 / 3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(12), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${12 / 3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(12), createInt(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${12 / -3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(-12), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-12 / 3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(-12), createInt(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-12 / -3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createInt(11), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${Math.floor(11 / 3)}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Float, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(0), createFloat(0)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Divide(0.0, 0.0)> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(3), createFloat(0)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Divide(3.0, 0.0)> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(0), createFloat(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${0 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(3), createFloat(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(12), createFloat(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${12 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(12), createFloat(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${12 / -3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(-12), createFloat(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-12 / -3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(-12), createFloat(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-12 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(-12), createFloat(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-12 / -3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(3.142), createFloat(2.718)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 / 2.718}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(3.142), createFloat(-2.718)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 / -2.718}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(2.718), createFloat(-3.142)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${2.718 / -3.142}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(-2.718), createFloat(3.142)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-2.718 / 3.142}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Divide), createPair(createFloat(-2.718), createFloat(-3.142)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-2.718 / -3.142}`);
        assert.strictEqual(format(dependencies), 'NULL');
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Divide),
          createPair(createInt(3), createFloat(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Divide),
          createPair(createFloat(3), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 / 3}.0`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Divide),
          createPair(createInt(3), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 / 3.142}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Divide),
          createPair(createFloat(3.142), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 / 3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
