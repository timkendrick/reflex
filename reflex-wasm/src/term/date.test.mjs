// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Date', (test) => {
    test('display', (assert, { createDate, display }) => {
      const timestamp = Date.now();
      assert.strictEqual(display(createDate(timestamp)), new Date(timestamp).toISOString());
    });

    test('format', (assert, { createDate, format }) => {
      const timestamp = Date.now();
      assert.strictEqual(
        format(createDate(timestamp)),
        `Date(${new Date(timestamp).toISOString()})`,
      );
    });

    test('hash', (assert, { createDate, hash }) => {
      const timestamp = Date.now();
      assert.strictEqual(hash(createDate(timestamp)), hash(createDate(timestamp)));
      assert.notStrictEqual(hash(createDate(timestamp)), hash(createDate(timestamp - 1)));
    });

    test('equals', (assert, { createDate, equals }) => {
      const timestamp = Date.now();
      assert.strictEqual(equals(createDate(timestamp), createDate(timestamp)), true);
      assert.strictEqual(equals(createDate(timestamp), createDate(timestamp - 1)), false);
    });
  });
};
