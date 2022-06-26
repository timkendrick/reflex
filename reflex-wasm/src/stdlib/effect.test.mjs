// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Effect', (test) => {
    test('(Symbol, List)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTriple,
      createSymbol,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Effect),
          createTriple(createSymbol(123), createInt(3), createSymbol(456)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3:Symbol(456)> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3:Symbol(456)> . NULL)');
      })();
    });
  });
};
