// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Split', (test) => {
    test('(String, String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString(''), createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[""]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["f", "o", "o"]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["", ""]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('oo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["f", ""]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('o')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["f", "", ""]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('f')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["", "oo"]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('fo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["", "o"]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Split),
          createPair(createString('foo'), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["foo"]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
