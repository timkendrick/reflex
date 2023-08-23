// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Add', (test) => {
    test('(Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(3), createInt(-1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + -1}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(3), createInt(-4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + -4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(-3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-3 + 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(-3), createInt(-4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-3 + -4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Float, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3), createFloat(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3), createFloat(-1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + -1}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3), createFloat(-4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + -4}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(-3), createFloat(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-3 + 4}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(-3), createFloat(-4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-3 + -4}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3.142), createFloat(2.718)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 + 2.718}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3.142), createFloat(-2.718)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 + -2.718}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(2.718), createFloat(-3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${2.718 + -3.142}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(-2.718), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-2.718 + 3.142}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(-2.718), createFloat(-3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${-2.718 + -3.142}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(0), createFloat(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${0 + 0}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(0), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${0 + 0}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(3), createFloat(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 3}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 3}.0`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createInt(3), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 3.142}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Add),
          createPair(createFloat(3.142), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3.142 + 3}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
