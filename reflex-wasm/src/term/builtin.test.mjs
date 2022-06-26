// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Builtin', (test) => {
    test('format', (assert, { createBuiltin, format, Stdlib }) => {
      assert.strictEqual(format(createBuiltin(Stdlib.Add)), 'Add');
    });

    test('hash', (assert, { createBuiltin, hash, Stdlib }) => {
      assert.strictEqual(hash(createBuiltin(Stdlib.Add)), hash(createBuiltin(Stdlib.Add)));
      assert.strictEqual(
        hash(createBuiltin(Stdlib.Subtract)),
        hash(createBuiltin(Stdlib.Subtract)),
      );
      assert.notStrictEqual(hash(createBuiltin(Stdlib.Add)), hash(createBuiltin(Stdlib.Subtract)));
    });

    test('equals', (assert, { createBuiltin, equals, Stdlib }) => {
      assert.strictEqual(equals(createBuiltin(Stdlib.Add), createBuiltin(Stdlib.Add)), true);
      assert.strictEqual(
        equals(createBuiltin(Stdlib.Subtract), createBuiltin(Stdlib.Subtract)),
        true,
      );
      assert.strictEqual(equals(createBuiltin(Stdlib.Add), createBuiltin(Stdlib.Subtract)), false);
    });

    test('arity', (assert, { createBuiltin, arity, Stdlib }) => {
      assert.strictEqual(arity(createBuiltin(Stdlib.Abs)), 1);
      assert.strictEqual(arity(createBuiltin(Stdlib.Add)), 2);
      assert.strictEqual(arity(createBuiltin(Stdlib.Fold)), 3);
    });

    test('builtin function applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(createInt(3), createInt(4)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4}`);
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};
