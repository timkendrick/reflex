// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Keys', (test) => {
    test('(List)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Keys), createUnitList(createEmptyList())),
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
              createBuiltin(Stdlib.Keys),
              createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[0, 1, 2]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createRecord,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Keys),
              createUnitList(createRecord(createEmptyList(), createEmptyList())),
            ),
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
              createBuiltin(Stdlib.Keys),
              createUnitList(
                createRecord(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createTriple(createInt(6), createInt(7), createInt(8)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
      createUnitList,
      evaluate,
      format,
      getListItems,
      getListLength,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Keys), createUnitList(createHashmap([]))),
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
              createBuiltin(Stdlib.Keys),
              createUnitList(
                createHashmap([
                  [createInt(3), createInt(6)],
                  [createInt(4), createInt(7)],
                  [createInt(5), createInt(8)],
                ]),
              ),
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
