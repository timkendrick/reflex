// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Variable', (test) => {
    test('format', (assert, { createVariable, format }) => {
      assert.strictEqual(format(createVariable(0)), 'Variable(0)');
      assert.strictEqual(format(createVariable(3)), 'Variable(3)');
      assert.strictEqual(format(createVariable(2 ** 32 - 1)), 'Variable(4294967295)');
    });

    test('hash', (assert, { createVariable, hash }) => {
      assert.strictEqual(hash(createVariable(0)), hash(createVariable(0)));
      assert.strictEqual(hash(createVariable(3)), hash(createVariable(3)));
      assert.strictEqual(
        new Set(Array.from({ length: 20 }).map((_, i) => hash(createVariable(i)))).size,
        20,
      );
    });

    test('equals', (assert, { createVariable, equals }) => {
      assert.strictEqual(equals(createVariable(0), createVariable(0)), true);
      assert.strictEqual(equals(createVariable(3), createVariable(3)), true);
    });
  });
};
