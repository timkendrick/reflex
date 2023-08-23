// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveTree', (test) => {
    test('(Tree)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTree,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveTree),
          createUnitList(createTree(NULL, NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(NULL . NULL)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveTree),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveTree),
          createUnitList(
            createTree(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveTree),
          createUnitList(
            createTree(
              createTree(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
              ),
              createTree(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '((Abs(-1) . Abs(-2)) . (Abs(-3) . Abs(-4)))');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
