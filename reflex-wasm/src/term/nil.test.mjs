// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Nil', (test) => {
    test('format', (assert, { createNil, format }) => {
      assert.strictEqual(format(createNil()), 'null');
    });

    test('hash', (assert, { createNil, hash }) => {
      assert.strictEqual(hash(createNil()), hash(createNil()));
    });

    test('equals', (assert, { createNil, equals }) => {
      assert.strictEqual(equals(createNil(), createNil()), true);
    });

    test('toJson', (assert, { createNil, getStringValue, toJson }) => {
      assert.strictEqual(getStringValue(toJson(createNil())), 'null');
    });
  });
};
