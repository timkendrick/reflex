// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Scan', (test) => {
    test('(Application, Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Scan),
        createTriple(
          createApplication(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
          createInt(5),
          createBuiltin(Stdlib.Add),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:"reflex::scan":[Add(3, 4), 5, Add]:null>}',
      );
      assert.strictEqual(
        format(dependencies),
        '(<CustomCondition:"reflex::scan":[Add(3, 4), 5, Add]:null> . NULL)',
      );
    });
  });
};
