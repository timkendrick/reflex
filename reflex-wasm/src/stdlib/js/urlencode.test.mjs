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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('   ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('   ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('  foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('  foo')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo  ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo  ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('  foo  ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('  foo  ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo bar')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString(' foo bar ')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent(' foo bar ')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('foo bar baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('foo bar baz')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Urlencode),
          createUnitList(createString('#$%&+,/:;=?@[]')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), JSON.stringify(encodeURIComponent('#$%&+,/:;=?@[]')));
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
