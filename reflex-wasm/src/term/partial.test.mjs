// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Partial', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
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
