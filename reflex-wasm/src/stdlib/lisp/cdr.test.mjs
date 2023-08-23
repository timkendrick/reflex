// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Cdr', (test) => {
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
          createBuiltin(Stdlib.Cdr),
          createUnitList(createTree(NULL, NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cdr),
          createUnitList(createTree(createInt(3), NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cdr),
          createUnitList(createTree(NULL, createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cdr),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Cdr),
          createUnitList(createTree(createInt(3), createTree(createInt(4), NULL))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(4 . NULL)');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
