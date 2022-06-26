// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectHashmap', (test) => {
    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
      createInt,
      createPair,
      createString,
      createRangeIterator,
      createTriple,
      createUnitList,
      createZipIterator,
      evaluate,
      format,
      getHashmapValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createUnitList(createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map({0})');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createUnitList(
            createZipIterator(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createRangeIterator(3, 3),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map({3})');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectHashmap),
          createUnitList(
            createTriple(
              createString('foo'),
              createPair(createString('bar'), createInt(4)),
              createString('baz'),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{((<TypeError:List:"foo"> . NULL) . (<TypeError:List:"baz"> . NULL))}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
