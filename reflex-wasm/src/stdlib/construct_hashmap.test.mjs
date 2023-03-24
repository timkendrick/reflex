// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ConstructHashmap', (test) => {
    test('(List, List)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createPair,
      createString,
      createTriple,
      evaluate,
      format,
      getHashmapValue,
      hasHashmapKey,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashmap),
          createPair(createEmptyList(), createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(0)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashmap),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(3)');
        assert.strictEqual(hasHashmapKey(result, createString('foo')), true);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(hasHashmapKey(result, createString('bar')), true);
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(hasHashmapKey(result, createString('baz')), true);
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(hasHashmapKey(result, createString('qux')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ConstructHashmap),
          createPair(
            createPair(createString('foo'), createString('bar')),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(2)');
        assert.strictEqual(hasHashmapKey(result, createString('foo')), true);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(hasHashmapKey(result, createString('bar')), true);
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(hasHashmapKey(result, createString('baz')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
