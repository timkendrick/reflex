// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_If', (test) => {
    test('(Boolean, Lambda, Lambda)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createInt,
      createLambda,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(
            createBoolean(false),
            createLambda(0, createInt(3)),
            createLambda(0, createInt(4)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.If),
          createTriple(
            createBoolean(true),
            createLambda(0, createInt(3)),
            createLambda(0, createInt(4)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
