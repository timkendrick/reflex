// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Condition', (test) => {
    test('format', (assert, {
      createBuiltin,
      createCustomCondition,
      createErrorCondition,
      createEmptyList,
      createInt,
      createInvalidFunctionArgsCondition,
      createInvalidFunctionTargetCondition,
      createSymbol,
      createString,
      createTriple,
      createTypeErrorCondition,
      createUnitList,
      format,
      NULL,
      Stdlib,
    }) => {
      assert.strictEqual(
        format(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        '<CustomCondition:Symbol(123):3:Symbol(0)>',
      );
      assert.strictEqual(
        format(createErrorCondition(createString('foo'))),
        '<ErrorCondition:"foo">',
      );
      assert.strictEqual(
        format(createTypeErrorCondition(NULL, createString('foo'))),
        '<TypeErrorCondition:<unknown>:"foo">',
      );
      assert.strictEqual(
        format(createInvalidFunctionTargetCondition(createString('foo'))),
        '<InvalidFunctionTargetCondition:"foo">',
      );
      assert.strictEqual(
        format(createInvalidFunctionArgsCondition(createBuiltin(Stdlib.Add), createEmptyList())),
        '<InvalidFunctionArgsCondition:Add()>',
      );
      assert.strictEqual(
        format(
          createInvalidFunctionArgsCondition(
            createBuiltin(Stdlib.Add),
            createUnitList(createInt(3)),
          ),
        ),
        '<InvalidFunctionArgsCondition:Add(3)>',
      );
      assert.strictEqual(
        format(
          createInvalidFunctionArgsCondition(
            createBuiltin(Stdlib.Add),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        ),
        '<InvalidFunctionArgsCondition:Add(3, 4, 5)>',
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
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
      );
      assert.notStrictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        hash(createCustomCondition(createSymbol(456), createInt(3), createSymbol(0))),
      );
      assert.notStrictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        hash(createCustomCondition(createSymbol(123), createInt(4), createSymbol(0))),
      );
      assert.notStrictEqual(
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        hash(createCustomCondition(createSymbol(123), createInt(3), createSymbol(1))),
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
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
          createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
          createCustomCondition(createSymbol(123), createInt(4), createSymbol(0)),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(1)),
        ),
        false,
      );
    });
  });
};
