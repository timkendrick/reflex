// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Partial', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });

    test('arity', (assert, {
      createBoolean,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createPair,
      createPartial,
      createTriple,
      createUnitList,
      arity,
      Stdlib,
      FALSE,
      TRUE,
    }) => {
      assert.deepEqual(
        arity(createPartial(createLambda(2, createBoolean(true)), createEmptyList())),
        [2, FALSE],
      );
      assert.deepEqual(
        arity(createPartial(createLambda(2, createBoolean(true)), createUnitList(createInt(3)))),
        [1, FALSE],
      );
      assert.deepEqual(
        arity(
          createPartial(
            createLambda(2, createBoolean(true)),
            createPair(createInt(3), createInt(4)),
          ),
        ),
        [0, FALSE],
      );
      assert.deepEqual(
        arity(
          createPartial(
            createLambda(2, createBoolean(true)),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        ),
        [0, FALSE],
      );
      assert.deepEqual(arity(createPartial(createBuiltin(Stdlib.Add), createEmptyList())), [
        2,
        FALSE,
      ]);
      assert.deepEqual(
        arity(createPartial(createBuiltin(Stdlib.Add), createUnitList(createInt(3)))),
        [1, FALSE],
      );
      assert.deepEqual(
        arity(createPartial(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4)))),
        [0, FALSE],
      );
      assert.deepEqual(
        arity(
          createPartial(
            createBuiltin(Stdlib.Add),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        ),
        [0, FALSE],
      );
      );
    });

    test('partial function applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPartial,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createPartial(createBuiltin(Stdlib.Add), createUnitList(createInt(3))),
        createUnitList(createInt(4)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4}`);
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};
