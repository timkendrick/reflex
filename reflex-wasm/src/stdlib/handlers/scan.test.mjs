// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Scan', (test) => {
    test('(Application, Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
      createPair,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Scan),
        createTriple(
          createLambda(
            0,
            createApplication(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
          ),
          createInt(5),
          createBuiltin(Stdlib.Add),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:"reflex::scan":[(0) => Add(3, 4), 5, Add]:null>}',
      );
      assert.deepEqual(
        getStateDependencies(dependencies).map((dependency) => format(dependency)),
        ['<CustomCondition:"reflex::scan":[(0) => Add(3, 4), 5, Add]:null>'],
      );
    });
  });
};
