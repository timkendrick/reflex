// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::FilterIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createBoolean,
      createEmptyIterator,
      createBuiltin,
      createList,
      createNil,
      createFilterIterator,
      createRangeIterator,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(createFilterIterator(createEmptyIterator(), createBuiltin(Stdlib.Not))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createFilterIterator(createRangeIterator(3, 3), createBuiltin(Stdlib.Not)),
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
            createFilterIterator(
              createList([createNil(), createBoolean(false), createBoolean(true), createNil()]),
              createBuiltin(Stdlib.Not),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[null, false, null]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
