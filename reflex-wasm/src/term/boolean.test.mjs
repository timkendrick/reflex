// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Boolean', (test) => {
    test('format', (assert, { createBoolean, format }) => {
      assert.strictEqual(format(createBoolean(true)), 'true');
      assert.strictEqual(format(createBoolean(false)), 'false');
    });

    test('hash', (assert, { createBoolean, hash }) => {
      assert.strictEqual(hash(createBoolean(true)), hash(createBoolean(true)));
      assert.strictEqual(hash(createBoolean(false)), hash(createBoolean(false)));
      assert.notStrictEqual(hash(createBoolean(true)), hash(createBoolean(false)));
    });

    test('equals', (assert, { createBoolean, equals }) => {
      assert.strictEqual(equals(createBoolean(true), createBoolean(true)), true);
      assert.strictEqual(equals(createBoolean(false), createBoolean(false)), true);
      assert.strictEqual(equals(createBoolean(true), createBoolean(false)), false);
    });

    test('toJson', (assert, { createBoolean, getStringValue, toJson }) => {
      assert.strictEqual(getStringValue(toJson(createBoolean(false))), 'false');
      assert.strictEqual(getStringValue(toJson(createBoolean(true))), 'true');
    });
  });
};
