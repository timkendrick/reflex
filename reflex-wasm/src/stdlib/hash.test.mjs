// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Hash', (test) => {
    test('()', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      evaluate,
      format,
      getStateDependencies,
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Hash), createEmptyList());
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(hash(createEmptyList()) & BigInt(0x00000000ffffffff))})`,
        );
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
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(hash(createUnitList(createInt(0))) & BigInt(0x00000000ffffffff))})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(hash(createUnitList(createInt(3))) & BigInt(0x00000000ffffffff))})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createUnitList(createInt(-3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(hash(createUnitList(createInt(-3))) & BigInt(0x00000000ffffffff))})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createUnitList(createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(hash(createUnitList(createString(''))) & BigInt(0x00000000ffffffff))})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(
            hash(createUnitList(createString('foo'))) & BigInt(0x00000000ffffffff),
          )})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Int, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      hash,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Hash),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          `Symbol(${Number(
            hash(createTriple(createInt(3), createInt(4), createInt(5))) &
              BigInt(0x00000000ffffffff),
          )})`,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
