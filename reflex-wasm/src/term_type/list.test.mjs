// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::List', (test) => {
    test('format', (assert, {
      createEmptyList,
      createInt,
      createList,
      createPair,
      createTriple,
      createUnitList,
      format,
    }) => {
      assert.strictEqual(format(createEmptyList()), '[]');
      assert.strictEqual(format(createUnitList(createInt(3))), '[3]');
      assert.strictEqual(format(createPair(createInt(3), createInt(4))), '[3, 4]');
      assert.strictEqual(
        format(createTriple(createInt(3), createInt(4), createInt(5))),
        '[3, 4, 5]',
      );
      assert.strictEqual(
        format(createList([createInt(1), createInt(2), createInt(3), createInt(4), createInt(5)])),
        '[1, 2, 3, 4, 5]',
      );
    });

    test('hash', (assert, {
      createEmptyList,
      createInt,
      createList,
      createPair,
      createTriple,
      createUnitList,
      hash,
    }) => {
      assert.strictEqual(hash(createEmptyList()), hash(createEmptyList()));
      assert.strictEqual(hash(createUnitList(createInt(3))), hash(createUnitList(createInt(3))));
      assert.strictEqual(
        hash(createPair(createInt(3), createInt(4))),
        hash(createPair(createInt(3), createInt(4))),
      );
      assert.strictEqual(
        hash(createTriple(createInt(3), createInt(4), createInt(5))),
        hash(createTriple(createInt(3), createInt(4), createInt(5))),
      );
      assert.strictEqual(
        hash(createList([createInt(1), createInt(2), createInt(3), createInt(4), createInt(5)])),
        hash(createList([createInt(1), createInt(2), createInt(3), createInt(4), createInt(5)])),
      );
      assert.notStrictEqual(
        hash(createTriple(createInt(3), createInt(4), createInt(5))),
        hash(createTriple(createInt(3), createInt(4), createInt(6))),
      );
      assert.notStrictEqual(
        hash(createTriple(createInt(3), createInt(4), createInt(5))),
        hash(createPair(createInt(3), createInt(4))),
      );
      assert.notStrictEqual(
        hash(createPair(createInt(3), createInt(4))),
        hash(createTriple(createInt(3), createInt(4), createInt(5))),
      );
    });

    test('equals', (assert, {
      createEmptyList,
      createInt,
      createList,
      createPair,
      createTriple,
      createUnitList,
      equals,
    }) => {
      assert.strictEqual(equals(createEmptyList(), createEmptyList()), true);
      assert.strictEqual(equals(createUnitList(createInt(3)), createUnitList(createInt(3))), true);
      assert.strictEqual(
        equals(createPair(createInt(3), createInt(4)), createPair(createInt(3), createInt(4))),
        true,
      );
      assert.strictEqual(
        equals(
          createTriple(createInt(3), createInt(4), createInt(5)),
          createTriple(createInt(3), createInt(4), createInt(5)),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createTriple(createInt(3), createInt(4), createInt(5)),
          createTriple(createInt(3), createInt(4), createInt(6)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createList([createInt(1), createInt(2), createInt(3), createInt(4), createInt(5)]),
          createList([createInt(1), createInt(2), createInt(3), createInt(4), createInt(5)]),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createTriple(createInt(3), createInt(4), createInt(5)),
          createPair(createInt(3), createInt(4)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createPair(createInt(3), createInt(4)),
          createTriple(createInt(3), createInt(4), createInt(5)),
        ),
        false,
      );
    });

    test('basic item access', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const list = createTriple(createInt(3), createInt(4), createInt(5));
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(list, createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(list, createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(list, createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('iteration', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
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
          createUnitList(
            createApplication(createBuiltin(Stdlib.Iterate), createUnitList(createEmptyList())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTriple(createInt(1), createInt(2), createInt(3))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[1, 2, 3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
