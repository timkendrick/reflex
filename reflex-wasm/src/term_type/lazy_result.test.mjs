// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Effect', (test) => {
    test('format', (assert, {
      createCustomCondition,
      createLazyResult,
      createInt,
      createSymbol,
      createTree,
      format,
      NULL,
    }) => {
      assert.strictEqual(
        format(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        '<lazy:3:(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)>',
      );
    });

    test('hash', (assert, {
      createCustomCondition,
      createLazyResult,
      createInt,
      createSymbol,
      createTree,
      hash,
      NULL,
    }) => {
      assert.strictEqual(
        hash(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        hash(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
      );
      assert.notStrictEqual(
        hash(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        hash(
          createLazyResult(
            createInt(4),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
      );
      assert.notStrictEqual(
        hash(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        hash(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
      );
    });

    test('equals', (assert, {
      createCustomCondition,
      createLazyResult,
      createInt,
      createSymbol,
      createTree,
      equals,
      NULL,
    }) => {
      assert.strictEqual(
        equals(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
          createLazyResult(
            createInt(4),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
          createLazyResult(
            createInt(3),
            createTree(
              createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)),
              NULL,
            ),
          ),
        ),
        false,
      );
    });

    test('evaluate', (assert, {
      createCustomCondition,
      createLazyResult,
      createHashmap,
      createInt,
      createSymbol,
      createString,
      createTree,
      evaluate,
      format,
      NULL,
    }) => {
      (() => {
        const expression = createLazyResult(
          createString('foo'),
          createTree(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), NULL),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
      (() => {
        const expression = createLazyResult(
          createString('foo'),
          createTree(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), NULL),
        );
        const [result, dependencies] = evaluate(expression, createHashmap([]));
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
      (() => {
        const expression = createLazyResult(
          createString('foo'),
          createTree(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), NULL),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(123), createInt(4), createSymbol(0)), createInt(4)],
            [createCustomCondition(createSymbol(456), createInt(3), createSymbol(0)), createInt(5)],
          ]),
        );
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(
          format(dependencies),
          '(<CustomCondition:Symbol(123):3:Symbol(0)> . NULL)',
        );
      })();
    });
  });
};
