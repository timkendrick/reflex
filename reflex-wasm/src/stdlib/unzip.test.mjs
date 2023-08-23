// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Unzip', (test) => {
    test('(Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createPair,
      createRangeIterator,
      createString,
      createTriple,
      createUnitList,
      createZipIterator,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Unzip), createUnitList(createEmptyIterator())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[], []]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Unzip),
              createUnitList(
                createTriple(
                  createPair(createString('foo'), createInt(3)),
                  createPair(createString('bar'), createInt(4)),
                  createPair(createString('baz'), createInt(5)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[["foo", "bar", "baz"], [3, 4, 5]]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Unzip),
              createUnitList(
                createZipIterator(createRangeIterator(3, 3), createRangeIterator(6, 3)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[3, 4, 5], [6, 7, 8]]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
