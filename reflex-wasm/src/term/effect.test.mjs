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
        format(createEffect(createCustomCondition(createSymbol(123), createInt(3)))),
        '(!<Custom:Symbol(123):3>)',
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
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3)))),
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3)))),
      );
      assert.notStrictEqual(
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(3)))),
        hash(createEffect(createCustomCondition(createSymbol(123), createInt(4)))),
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
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
          createEffect(createCustomCondition(createSymbol(123), createInt(4))),
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
        const expression = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const [result, dependencies] = evaluate(expression, createHashmap([]));
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(4)), createInt(4)],
            [createCustomCondition(createSymbol(456), createInt(3)), createInt(5)],
          ]),
        );
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createInt(3)],
            [createCustomCondition(createSymbol(123), createInt(4)), createInt(4)],
            [createCustomCondition(createSymbol(456), createInt(3)), createInt(5)],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
    });
  });
};
