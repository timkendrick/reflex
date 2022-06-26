// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Effect', (test) => {
    test('format', (assert, {
      createCustomCondition,
      createEffect,
      createInt,
      createSymbol,
      format,
    }) => {
      assert.strictEqual(
        format(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        ),
        '<!<CustomCondition:Symbol(123):3:Symbol(0)>>',
      );
    });

    test('hash', (assert, {
      createCustomCondition,
      createEffect,
      createInt,
      createSymbol,
      hash,
    }) => {
      assert.strictEqual(
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)))),
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)))),
      );
      assert.notStrictEqual(
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)))),
        hash(createEffect(createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)))),
      );
      assert.notStrictEqual(
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)))),
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(4), createSymbol(0)))),
      );
      assert.notStrictEqual(
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)))),
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(1)))),
      );
    });

    test('equals', (assert, {
      createCustomCondition,
      createEffect,
      createInt,
      createSymbol,
      equals,
    }) => {
      assert.strictEqual(
        equals(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(456), createInt(3), createSymbol(0))),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(123), createInt(4), createSymbol(0))),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(1))),
        ),
        false,
      );
    });

    test('evaluate', (assert, {
      createCustomCondition,
      createEffect,
      createHashmap,
      createInt,
      createSymbol,
      evaluate,
      format,
      NULL,
    }) => {
      (() => {
        const expression = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):3:Symbol(0)>}');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
      (() => {
        const expression = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const [result, dependencies] = evaluate(expression, createHashmap([]));
        assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):3:Symbol(0)>}');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
      (() => {
        const expression = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(4), createSymbol(0)), createInt(4)],
            [createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)), createInt(5)],
          ]),
        );
        assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):3:Symbol(0)>}');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
      (() => {
        const expression = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(123), createInt(4), createSymbol(0)), createInt(4)],
            [createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)), createInt(5)],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
    });
  });
};
