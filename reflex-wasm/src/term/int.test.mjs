// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Int', (test) => {
    test('format', (assert, { createInt, format }) => {
      assert.strictEqual(format(createInt(0)), '0');
      assert.strictEqual(format(createInt(3)), '3');
      assert.strictEqual(format(createInt(-3)), '-3');
    });

    test('hash', (assert, { createInt, hash }) => {
      assert.strictEqual(hash(createInt(0)), hash(createInt(0)));
      assert.strictEqual(hash(createInt(3)), hash(createInt(3)));
      assert.strictEqual(hash(createInt(-3)), hash(createInt(-3)));
      assert.notStrictEqual(hash(createInt(-3)), hash(createInt(3)));
      assert.strictEqual(
        new Set(Array.from({ length: 20 }).map((_, i) => hash(createInt(i)))).size,
        20,
      );
    });

    test('equals', (assert, { createInt, equals }) => {
      assert.strictEqual(equals(createInt(0), createInt(0)), true);
      assert.strictEqual(equals(createInt(3), createInt(3)), true);
      assert.strictEqual(equals(createInt(-3), createInt(-3)), true);
      assert.strictEqual(equals(createInt(-3), createInt(3)), false);
    });
  });
};
