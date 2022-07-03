// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::IntersperseIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createEmptyIterator,
      createEvaluateIterator,
      createBuiltin,
      createInt,
      createIntersperseIterator,
      createPair,
      createString,
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
              createIntersperseIterator(createEmptyIterator(), createString('foo')),
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
              createIntersperseIterator(createUnitList(createInt(3)), createString('foo')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        debugger;
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createEvaluateIterator(
              createIntersperseIterator(
                createPair(createInt(3), createInt(4)),
                createString('foo'),
              ),
            ),
          ),
        );
        debugger;
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, "foo", 4]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createEvaluateIterator(
              createIntersperseIterator(
                createTriple(createInt(3), createInt(4), createInt(5)),
                createString('foo'),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, "foo", 4, "foo", 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
