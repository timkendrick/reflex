// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Nil', (test) => {
    test('display', (assert, { createNil, display }) => {
      assert.strictEqual(display(createNil()), 'null');
    });

    test('format', (assert, { createNil, format }) => {
      assert.strictEqual(format(createNil()), 'null');
    });

    test('hash', (assert, { createNil, hash }) => {
      assert.strictEqual(hash(createNil()), hash(createNil()));
    });

    test('equals', (assert, { createNil, equals }) => {
      assert.strictEqual(equals(createNil(), createNil()), true);
    });
  });
};
