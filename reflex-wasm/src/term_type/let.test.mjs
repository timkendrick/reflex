// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Let', (test) => {
    test('format', (assert, { createLet, createBoolean, createInt, format }) => {
      assert.strictEqual(format(createLet(createInt(3), createBoolean(true))), '{let 3; true}');
      assert.strictEqual(format(createLet(createInt(3), createBoolean(true))), '{let 3; true}');
    });

    test('hash', (assert, { createBoolean, createInt, createLet, hash }) => {
      assert.strictEqual(
        hash(createLet(createInt(3), createBoolean(true))),
        hash(createLet(createInt(3), createBoolean(true))),
      );
      assert.strictEqual(
        hash(createLet(createInt(4), createBoolean(true))),
        hash(createLet(createInt(4), createBoolean(true))),
      );
      assert.notStrictEqual(
        hash(createLet(createInt(3), createBoolean(true))),
        hash(createLet(createInt(4), createBoolean(true))),
      );
      assert.notStrictEqual(
        hash(createLet(createInt(3), createBoolean(true))),
        hash(createLet(createInt(3), createBoolean(false))),
      );
    });

    test('equals', (assert, { createBoolean, createInt, createLet, equals }) => {
      assert.strictEqual(
        equals(
          createLet(createInt(3), createBoolean(true)),
          createLet(createInt(3), createBoolean(true)),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createLet(createInt(4), createBoolean(true)),
          createLet(createInt(4), createBoolean(true)),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createLet(createInt(3), createBoolean(true)),
          createLet(createInt(4), createBoolean(true)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createLet(createInt(3), createBoolean(true)),
          createLet(createInt(3), createBoolean(false)),
        ),
        false,
      );
    });

    test('basic let expressions', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLet,
      createPair,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createLet(createInt(3), createInt(4));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createLet(createInt(3), createVariable(0));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createLet(
          createInt(3),
          createApplication(createBuiltin(Stdlib.Add), createPair(createVariable(0), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('nested let expressions', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLet,
      createPair,
      createVariable,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createLet(
          createInt(3),
          createLet(
            createInt(4),
            createApplication(
              createBuiltin(Stdlib.Subtract),
              createPair(createVariable(1), createVariable(0)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createLet(
          createInt(3),
          createLet(
            createInt(4),
            createLet(
              createVariable(1),
              createLet(
                createVariable(1),
                createApplication(
                  createBuiltin(Stdlib.Subtract),
                  createPair(createVariable(1), createVariable(0)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
