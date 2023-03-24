// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('List performance', (_, perf) => {
    transformList(perf, 1000);
    transformList(perf, 10000);
    transformList(perf, 100000);
    transformList(perf, 1000000);
    transformList(perf, 10000000);
  });

  function transformList(perf, numItems) {
    perf(
      `List map transform (${numItems} items)`,
      (
        bench,
        {
          createApplication,
          createBuiltin,
          createEvaluateIterator,
          createInt,
          createList,
          createMapIterator,
          createUnitList,
          evaluate,
          format,
          getListItem,
          getListLength,
          isList,
          NULL,
          Stdlib,
        },
      ) => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createEvaluateIterator(
              createMapIterator(
                createList(Array.from({ length: numItems }, (_, i) => createInt(-i))),
                createBuiltin(Stdlib.Abs),
              ),
            ),
          ),
        );
        return bench(
          () => evaluate(expression, NULL),
          ([result, dependencies], assert) => {
            assert.ok(isList(result)), assert.strictEqual(format(getListItem(result, 0)), '0');
            assert.strictEqual(getListLength(result), numItems);
            assert.strictEqual(format(getListItem(result, numItems - 1)), `${numItems - 1}`);
            assert.strictEqual(format(dependencies), 'NULL');
          },
        );
      },
    );
  }
};
