// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Fold', (test) => {
    test('(List, Builtin, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createEmptyList(), createBuiltin(Stdlib.Subtract), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createBuiltin(Stdlib.Subtract),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test.skip('(Record, Builtin, Int)', (assert, {}) => {
      throw new Error('Not yet implemented');
    });

    test.skip('(Hashmap, Builtin, Int)', (assert, {}) => {
      throw new Error('Not yet implemented');
    });

    test('(Tree, Builtin, Int)', (assert, {
      createApplication,
      createBuiltin,
      createTree,
      createInt,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(
            createTree(createInt(3), createInt(4)),
            createBuiltin(Stdlib.Subtract),
            createInt(5),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${5 - 3 - 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator, Builtin, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createRangeIterator,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createEmptyIterator(), createBuiltin(Stdlib.Subtract), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Fold),
          createTriple(createRangeIterator(3, 3), createBuiltin(Stdlib.Subtract), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
