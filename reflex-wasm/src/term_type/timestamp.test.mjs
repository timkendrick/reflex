// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Timestamp', (test) => {
    test('display', (assert, { createTimestamp, display }) => {
      const timestamp = Date.now();
      assert.strictEqual(display(createTimestamp(timestamp)), new Date(timestamp).toISOString());
    });

    test('format', (assert, { createTimestamp, format }) => {
      const timestamp = Date.now();
      assert.strictEqual(
        format(createTimestamp(timestamp)),
        `Timestamp(${new Date(timestamp).toISOString()})`,
      );
    });

    test('hash', (assert, { createTimestamp, hash }) => {
      const timestamp = Date.now();
      assert.strictEqual(hash(createTimestamp(timestamp)), hash(createTimestamp(timestamp)));
      assert.notStrictEqual(hash(createTimestamp(timestamp)), hash(createTimestamp(timestamp - 1)));
    });

    test('equals', (assert, { createTimestamp, equals }) => {
      const timestamp = Date.now();
      assert.strictEqual(equals(createTimestamp(timestamp), createTimestamp(timestamp)), true);
      assert.strictEqual(equals(createTimestamp(timestamp), createTimestamp(timestamp - 1)), false);
    });
  });
};
