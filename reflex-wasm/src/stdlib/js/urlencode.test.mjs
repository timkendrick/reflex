// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Urlencode', (test) => {
    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('   ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('   ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('  foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('  foo')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo  ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo  ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('  foo  ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('  foo  ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo bar')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo bar ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo bar ')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar baz')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('#$%&+,/:;=?@[]')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('#$%&+,/:;=?@[]')));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
