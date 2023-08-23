// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Hashmap', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });

    test('basic property access', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashmap = createHashmap([
        [createString('foo'), createInt(3)],
        [createString('bar'), createInt(4)],
        [createString('baz'), createInt(5)],
      ]);
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(hashmap, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(hashmap, createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(hashmap, createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('missing keys', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashmap = createHashmap([
        [createString('foo'), createInt(3)],
        [createString('bar'), createInt(4)],
        [createString('baz'), createInt(5)],
      ]);
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(hashmap, createString('missing')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'null');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('empty hashmap lookups', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashmap = createHashmap([]);
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(hashmap, createString('missing')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'null');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('substitution', (assert, {
      createApplication,
      createInt,
      createHashmap,
      createLambda,
      createString,
      createTriple,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      getHashmapEntries,
      NULL,
    }) => {
      (() => {
        const expression = createApplication(
          createLambda(
            3,
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
          ),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.deepEqual(
          new Map(getHashmapEntries(result).map(([key, value]) => [format(key), format(value)])),
          new Map([
            ['"foo"', '3'],
            ['"bar"', '4'],
            ['"baz"', '5'],
          ]),
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createLambda(
            3,
            createHashmap([
              [createString('foo'), createVariable(2)],
              [createString('bar'), createVariable(1)],
              [createString('baz'), createVariable(0)],
            ]),
          ),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.deepEqual(
          new Map(getHashmapEntries(result).map(([key, value]) => [format(key), format(value)])),
          new Map([
            ['"foo"', '3'],
            ['"bar"', '4'],
            ['"baz"', '5'],
          ]),
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('iteration', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
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
          createUnitList(createHashmap([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        const items = getListItems(result).map(format);
        assert.strictEqual(items.length, 6);
        const entries = items
          .reduce((items, item, index) => {
            if (index % 2 === 0) {
              items.push([item, null]);
            } else {
              items[(index - 1) / 2][1] = item;
            }
            return items;
          }, [])
          .map(([key, value]) => `[${key}, ${value}]`);
        assert.ok(entries.includes('["foo", 3]'));
        assert.ok(entries.includes('["bar", 4]'));
        assert.ok(entries.includes('["baz", 5]'));
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
