// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::HashmapValuesIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
      createHashmapValuesIterator,
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
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createHashmapValuesIterator(createHashmap([]))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createHashmapValuesIterator(
              createHashmap([
                [createString('foo'), createInt(3)],
                [createString('bar'), createInt(4)],
                [createString('baz'), createInt(5)],
              ]),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(`[${getListItems(result).map(format).sort().join(', ')}]`, '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
