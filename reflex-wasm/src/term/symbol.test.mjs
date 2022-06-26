// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Symbol', (test) => {
    test('format', (assert, { createSymbol, format }) => {
      assert.strictEqual(format(createSymbol(0)), 'Symbol(0)');
      assert.strictEqual(format(createSymbol(3)), 'Symbol(3)');
      assert.strictEqual(format(createSymbol(2 ** 32 - 1)), 'Symbol(4294967295)');
    });

    test('hash', (assert, { createSymbol, hash }) => {
      assert.strictEqual(hash(createSymbol(0)), hash(createSymbol(0)));
      assert.strictEqual(hash(createSymbol(3)), hash(createSymbol(3)));
      assert.strictEqual(
        new Set(Array.from({ length: 20 }).map((_, i) => hash(createSymbol(i)))).size,
        20,
      );
    });

    test('equals', (assert, { createSymbol, equals }) => {
      assert.strictEqual(equals(createSymbol(0), createSymbol(0)), true);
      assert.strictEqual(equals(createSymbol(3), createSymbol(3)), true);
    });
  });
};
