// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectHashmap', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(createBuiltin(Stdlib.CollectHashmap), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'Map(0)');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(String, Int, String, Int, String, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createList,
      createString,
      evaluate,
      format,
      getStateDependencies,
      getHashmapValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('baz'),
            createInt(5),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(3)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createList([
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
            createApplication(createLambda(0, createInt(5)), createEmptyList()),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(3)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('Duplicate keys', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createList,
      createString,
      evaluate,
      format,
      getStateDependencies,
      getHashmapValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('foo'),
            createInt(5),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(2)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '5');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('Trailing keys', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createList,
      createString,
      evaluate,
      format,
      getHashmapValue,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('baz'),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(2)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
