// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Hash', (test) => {
    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createUnitList(createInt(0)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `Symbol(${hash(createInt(0))})`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createUnitList(createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `Symbol(${hash(createInt(3))})`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createUnitList(createInt(-3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `Symbol(${hash(createInt(-3))})`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createUnitList(createString('')));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `Symbol(${hash(createString(''))})`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createUnitList(createString('foo')));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `Symbol(${hash(createString('foo'))})`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
