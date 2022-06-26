// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::FlattenIterator', (test) => {
    test('iteration', (assert, {
      createEmptyIterator,
      createBuiltin,
      createInt,
      createFlattenIterator,
      createApplication,
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
          createUnitList(createFlattenIterator(createEmptyIterator())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createFlattenIterator(
              createTriple(createEmptyIterator(), createEmptyIterator(), createEmptyIterator()),
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
            createFlattenIterator(
              createTriple(
                createTriple(createInt(3), createInt(4), createInt(5)),
                createTriple(createInt(6), createInt(7), createInt(8)),
                createTriple(createInt(9), createInt(10), createInt(11)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8, 9, 10, 11]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
