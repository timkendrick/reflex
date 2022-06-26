// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::ChainIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createChainIterator,
      createEmptyIterator,
      createBuiltin,
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
          createUnitList(createChainIterator([createEmptyIterator(), createEmptyIterator()])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(createChainIterator([createEmptyIterator(), createRangeIterator(3, 3)])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(createChainIterator([createRangeIterator(3, 3), createEmptyIterator()])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createChainIterator([createRangeIterator(3, 3), createRangeIterator(6, 3)]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6, 7, 8]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
