// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Compiled', (test) => {
    test('format', (assert, { createCompiled, format }) => {
      assert.strictEqual(format(createCompiled(123, 3)), '<compiled:3:123>');
    });

    test('hash', (assert, { createCompiled, hash }) => {
      assert.strictEqual(hash(createCompiled(123, 3)), hash(createCompiled(123, 3)));
      assert.notStrictEqual(hash(createCompiled(123, 3)), hash(createCompiled(123, 4)));
      assert.notStrictEqual(hash(createCompiled(123, 3)), hash(createCompiled(456, 3)));
    });

    test('equals', (assert, { createCompiled, equals }) => {
      assert.strictEqual(equals(createCompiled(123, 3), createCompiled(123, 3)), true);
      assert.strictEqual(equals(createCompiled(123, 3), createCompiled(123, 4)), false);
      assert.strictEqual(equals(createCompiled(123, 3), createCompiled(456, 3)), false);
    });

    test('arity', (assert, { createCompiled, arity }) => {
      assert.strictEqual(arity(createCompiled(123, 0)), 0);
      assert.strictEqual(arity(createCompiled(123, 1)), 1);
      assert.strictEqual(arity(createCompiled(123, 3)), 3);
      assert.strictEqual(arity(createCompiled(123, 0x7FFFFFFF)), 0x7FFFFFFF);
    });

    test('compiled function applications', (assert, {
      createApplication,
      createCompiled,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createCompiled(Stdlib.Add, 2),
        createPair(createInt(3), createInt(4)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4}`);
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};
