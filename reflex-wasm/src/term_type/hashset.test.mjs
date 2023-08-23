// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Hashset', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });

    test('value lookups', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashset = createHashset([
        createString('foo'),
        createString('bar'),
        createString('baz'),
      ]);
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashset, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashset, createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashset, createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashset, createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('empty hashset lookups', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashset = createHashset([]);
      const expression = createApplication(
        createBuiltin(Stdlib.Has),
        createPair(hashset, createString('foo')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'false');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('iteration', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      getListItems,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createHashset([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createHashset([createString('foo'), createString('bar'), createString('baz')]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo"]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
