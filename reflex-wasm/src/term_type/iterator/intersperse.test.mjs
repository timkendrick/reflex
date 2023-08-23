// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::IntersperseIterator', (test) => {
    test('iteration', (assert, {
      createApplication,
      createEmptyIterator,
      createBuiltin,
      createInt,
      createIntersperseIterator,
      createPair,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(createIntersperseIterator(createEmptyIterator(), createString('foo'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIntersperseIterator(createUnitList(createInt(3)), createString('foo')),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIntersperseIterator(createPair(createInt(3), createInt(4)), createString('foo')),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, "foo", 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createIntersperseIterator(
              createTriple(createInt(3), createInt(4), createInt(5)),
              createString('foo'),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, "foo", 4, "foo", 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
