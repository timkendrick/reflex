// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::HashmapKeysIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
      createHashmapKeysIterator,
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
          createUnitList(createHashmapKeysIterator(createHashmap([]))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createHashmapKeysIterator(
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
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo"]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
