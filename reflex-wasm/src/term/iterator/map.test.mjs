// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::MapIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createEmptyIterator,
      createEvaluateIterator,
      createBuiltin,
      createInt,
      createMapIterator,
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
            createEvaluateIterator(
              createMapIterator(createEmptyIterator(), createBuiltin(Stdlib.Abs)),
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
            createEvaluateIterator(
              createMapIterator(
                createTriple(createInt(-3), createInt(-4), createInt(-5)),
                createBuiltin(Stdlib.Abs),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
