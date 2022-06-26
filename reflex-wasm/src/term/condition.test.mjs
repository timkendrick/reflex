// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Condition', (test) => {
    test('format', (assert, { createCustomCondition, createInt, createSymbol, format }) => {
      assert.strictEqual(
        format(createCustomCondition(createSymbol(123), createInt(3))),
        '<Custom:Symbol(123):3>',
      );
    });

    test('hash', (assert, {
      createCustomCondition,
      createErrorCondition,
      createInt,
      createPendingCondition,
      createString,
      createSymbol,
      hash,
    }) => {
      assert.strictEqual(hash(createPendingCondition()), hash(createPendingCondition()));
      assert.strictEqual(
        hash(createErrorCondition(createString('foo'))),
        hash(createErrorCondition(createString('foo'))),
      );
      assert.notStrictEqual(
        hash(createErrorCondition(createString('foo'))),
        hash(createErrorCondition(createString('bar'))),
      );
      assert.strictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3))),
        hash(createCustomCondition(createSymbol(123), createInt(3))),
      );
      assert.notStrictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3))),
        hash(createCustomCondition(createSymbol(123), createInt(4))),
      );
      assert.notStrictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3))),
        hash(createCustomCondition(createSymbol(456), createInt(3))),
      );
    });

    test('equals', (assert, {
      createCustomCondition,
      createErrorCondition,
      createInt,
      createPendingCondition,
      createSymbol,
      createString,
      equals,
    }) => {
      assert.strictEqual(equals(createPendingCondition(), createPendingCondition()), true);
      assert.strictEqual(
        equals(
          createErrorCondition(createString('foo')),
          createErrorCondition(createString('foo')),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createErrorCondition(createString('foo')),
          createErrorCondition(createString('bar')),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3)),
          createCustomCondition(createSymbol(123), createInt(3)),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3)),
          createCustomCondition(createSymbol(123), createInt(4)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3)),
          createCustomCondition(createSymbol(456), createInt(3)),
        ),
        false,
      );
    });
  });
};
