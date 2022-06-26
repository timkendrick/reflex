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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(hashmap, createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(hashmap, createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('missing keys', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createNil,
      createPair,
      createString,
      evaluate,
      format,
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
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('empty hashmap lookups', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createPair,
      createString,
      evaluate,
      format,
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
      assert.strictEqual(format(dependencies), 'NULL');
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
      getListItems,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Iterate), createUnitList(createHashmap([]))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createHashmap([
                  [createString('foo'), createInt(3)],
                  [createString('bar'), createInt(4)],
                  [createString('baz'), createInt(5)],
                ]),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '[["bar", 4], ["baz", 5], ["foo", 3]]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
