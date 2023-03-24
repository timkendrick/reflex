// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectTree', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.CollectTree), createEmptyList());
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(NULL . NULL)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int, Int)', (assert, {
      createApplication,
      createInt,
      createBuiltin,
      createEmptyList,
      createLambda,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectTree),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(4 . (3 . NULL))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectTree),
          createPair(
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(4 . (3 . NULL))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createEmptyList,
      createLambda,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectTree),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(5 . (4 . (3 . NULL)))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectTree),
          createTriple(
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
            createApplication(createLambda(0, createInt(5)), createEmptyList()),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(5 . (4 . (3 . NULL)))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
