// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Application', (test) => {
    test.skip('format', (assert) => {
      // TODO: Test Application formatting
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      // TODO: Test Application hashing
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      // TODO: Test Application equality
      throw new Error('Not yet implemented');
    });

    test('builtin function applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(createInt(3), createInt(4)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4}`);
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('nested function applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Subtract),
        createPair(
          createApplication(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
          createInt(1),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4 - 1}`);
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('invalid target', (assert, {
      createApplication,
      createEmptyList,
      createInt,
      evaluate,
      format,
      NULL,
    }) => {
      const expression = createApplication(createInt(3), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{(<InvalidFunctionTarget:3> . NULL)}');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('insufficient args', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(createBuiltin(Stdlib.Add), createUnitList(createInt(3)));
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Add(3)> . NULL)}');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('short-circuit function target', (assert, {
      createApplication,
      createCustomCondition,
      createEmptyList,
      createSignal,
      createString,
      createSymbol,
      evaluate,
      format,
      NULL,
    }) => {
      const expression = createApplication(
        createSignal(createCustomCondition(createSymbol(123), createString('foo'))),
        createEmptyList(),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{(<Custom:Symbol(123):"foo"> . NULL)}');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('short-circuit single function arg', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createInt,
      createPair,
      createSignal,
      createString,
      createSymbol,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(
          createInt(3),
          createSignal(createCustomCondition(createSymbol(123), createString('foo'))),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{(<Custom:Symbol(123):"foo"> . NULL)}');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('short-circuit multiple function args', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createPair,
      createSignal,
      createString,
      createSymbol,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(
          createSignal(createCustomCondition(createSymbol(123), createString('foo'))),
          createSignal(createCustomCondition(createSymbol(456), createString('bar'))),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{((<Custom:Symbol(123):"foo"> . NULL) . (<Custom:Symbol(456):"bar"> . NULL))}',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('state dependencies', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createEffect,
      createHashmap,
      createInt,
      createPair,
      createSymbol,
      evaluate,
      format,
      Stdlib,
    }) => {
      (() => {
        const target = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const args = createPair(createInt(3), createInt(4));
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createBuiltin(Stdlib.Add)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const target = createBuiltin(Stdlib.Add);
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
          createInt(4),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([[createCustomCondition(createSymbol(123), createInt(3)), createInt(3)]]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const target = createBuiltin(Stdlib.Add);
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
          createEffect(createCustomCondition(createSymbol(234), createInt(4))),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createInt(3)],
            [createCustomCondition(createSymbol(234), createInt(4)), createInt(4)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(
          format(dependencies),
          '((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL))',
        );
      })();
      (() => {
        const target = createEffect(createCustomCondition(createSymbol(123), createInt(3)));
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(234), createInt(4))),
          createEffect(createCustomCondition(createSymbol(345), createInt(5))),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createBuiltin(Stdlib.Add)],
            [createCustomCondition(createSymbol(234), createInt(4)), createInt(3)],
            [createCustomCondition(createSymbol(345), createInt(5)), createInt(4)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(345):5> . NULL) . (<Custom:Symbol(234):4> . NULL)) . (<Custom:Symbol(123):3> . NULL))',
        );
      })();
      (() => {
        const expression = createApplication(
          createEffect(createCustomCondition(createSymbol(123), createInt(3))),
          createPair(
            createApplication(
              createEffect(createCustomCondition(createSymbol(234), createInt(4))),
              createPair(
                createEffect(createCustomCondition(createSymbol(345), createInt(5))),
                createEffect(createCustomCondition(createSymbol(456), createInt(6))),
              ),
            ),
            createApplication(
              createEffect(createCustomCondition(createSymbol(567), createInt(7))),
              createPair(
                createEffect(createCustomCondition(createSymbol(678), createInt(8))),
                createEffect(createCustomCondition(createSymbol(789), createInt(9))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createBuiltin(Stdlib.Add)],
            [createCustomCondition(createSymbol(234), createInt(4)), createBuiltin(Stdlib.Add)],
            [createCustomCondition(createSymbol(345), createInt(5)), createInt(3)],
            [createCustomCondition(createSymbol(456), createInt(6)), createInt(4)],
            [createCustomCondition(createSymbol(567), createInt(7)), createBuiltin(Stdlib.Add)],
            [createCustomCondition(createSymbol(678), createInt(8)), createInt(5)],
            [createCustomCondition(createSymbol(789), createInt(9)), createInt(6)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4 + 5 + 6}`);
        assert.strictEqual(
          format(dependencies),
          '(((((<Custom:Symbol(789):9> . NULL) . (<Custom:Symbol(678):8> . NULL)) . (<Custom:Symbol(567):7> . NULL)) . (((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . (<Custom:Symbol(234):4> . NULL))) . (<Custom:Symbol(123):3> . NULL))',
        );
      })();
    });
  });
};
