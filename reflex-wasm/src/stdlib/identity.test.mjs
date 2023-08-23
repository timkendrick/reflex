// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Identity', (test) => {
    test('(Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Identity),
          createUnitList(createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Identity),
          createUnitList(createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Identity),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Identity),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Identity),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
