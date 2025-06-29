// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Application', (test) => {
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(createInt(3), createInt(4)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), `${3 + 4}`);
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('nested function applications', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      getStateDependencies,
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
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('invalid target', (assert, {
      createApplication,
      createEmptyList,
      createInt,
      evaluate,
      format,
      getStateDependencies,
      NULL,
    }) => {
      const expression = createApplication(createInt(3), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<InvalidFunctionTargetCondition:3>}');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('insufficient args', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(createBuiltin(Stdlib.Add), createUnitList(createInt(3)));
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<InvalidFunctionArgsCondition:Add(3)>}');
      assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
    }) => {
      const expression = createApplication(
        createSignal(
          createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
        ),
        createEmptyList(),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):"foo":Symbol(0)>}');
      assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(
          createInt(3),
          createSignal(
            createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
          ),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):"foo":Symbol(0)>}');
      assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Add),
        createPair(
          createSignal(
            createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
          ),
          createSignal(
            createCustomCondition(createSymbol(456), createString('bar'), createSymbol(0)),
          ),
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{<CustomCondition:Symbol(123):"foo":Symbol(0)>,<CustomCondition:Symbol(456):"bar":Symbol(0)>}',
      );
      assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      Stdlib,
    }) => {
      (() => {
        const target = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const args = createPair(createInt(3), createInt(4));
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createBuiltin(Stdlib.Add),
            ],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
        );
      })();
      (() => {
        const target = createBuiltin(Stdlib.Add);
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createInt(4),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), createInt(3)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
        );
      })();
      (() => {
        const target = createBuiltin(Stdlib.Add);
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(234), createInt(4), createSymbol(0))),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)), createInt(4)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
        );
      })();
      (() => {
        const target = createEffect(
          createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
        );
        const args = createPair(
          createEffect(createCustomCondition(createSymbol(234), createInt(4), createSymbol(0))),
          createEffect(createCustomCondition(createSymbol(345), createInt(5), createSymbol(0))),
        );
        const expression = createApplication(target, args);
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createBuiltin(Stdlib.Add),
            ],
            [createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)), createInt(4)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(345):5:Symbol(0)>',
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
        );
      })();
      (() => {
        const expression = createApplication(
          createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
          createPair(
            createApplication(
              createEffect(createCustomCondition(createSymbol(234), createInt(4), createSymbol(0))),
              createPair(
                createEffect(
                  createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
                ),
                createEffect(
                  createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
                ),
              ),
            ),
            createApplication(
              createEffect(createCustomCondition(createSymbol(567), createInt(7), createSymbol(0))),
              createPair(
                createEffect(
                  createCustomCondition(createSymbol(678), createInt(8), createSymbol(0)),
                ),
                createEffect(
                  createCustomCondition(createSymbol(789), createInt(9), createSymbol(0)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createBuiltin(Stdlib.Add),
            ],
            [
              createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
              createBuiltin(Stdlib.Add),
            ],
            [createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)), createInt(4)],
            [
              createCustomCondition(createSymbol(567), createInt(7), createSymbol(0)),
              createBuiltin(Stdlib.Add),
            ],
            [createCustomCondition(createSymbol(678), createInt(8), createSymbol(0)), createInt(5)],
            [createCustomCondition(createSymbol(789), createInt(9), createSymbol(0)), createInt(6)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4 + 5 + 6}`);
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(789):9:Symbol(0)>',
            '<CustomCondition:Symbol(678):8:Symbol(0)>',
            '<CustomCondition:Symbol(567):7:Symbol(0)>',
            '<CustomCondition:Symbol(456):6:Symbol(0)>',
            '<CustomCondition:Symbol(345):5:Symbol(0)>',
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
        );
      })();
    });

    test('result caching', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createEffect,
      createHashmap,
      createInt,
      createPair,
      createString,
      createSymbol,
      evaluate,
      format,
      getStateDependencies,
      Stdlib,
      NULL,
    }) => {
      (() => {
        const target = createBuiltin(Stdlib.Add);
        const args = createPair(
          createEffect(
            createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
          ),
          createEffect(
            createCustomCondition(createSymbol(456), createString('bar'), createSymbol(0)),
          ),
        );
        const expression = createApplication(target, args);
        {
          const state = NULL;
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(
            format(result),
            '{<CustomCondition:Symbol(123):"foo":Symbol(0)>,<CustomCondition:Symbol(456):"bar":Symbol(0)>}',
          );
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = NULL;
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(
            format(result),
            '{<CustomCondition:Symbol(123):"foo":Symbol(0)>,<CustomCondition:Symbol(456):"bar":Symbol(0)>}',
          );
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), '{<CustomCondition:Symbol(456):"bar":Symbol(0)>}');
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), '{<CustomCondition:Symbol(456):"bar":Symbol(0)>}');
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
            [
              createCustomCondition(createSymbol(789), createString('baz'), createSymbol(0)),
              createInt(5),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), '{<CustomCondition:Symbol(456):"bar":Symbol(0)>}');
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = NULL;
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(
            format(result),
            '{<CustomCondition:Symbol(123):"foo":Symbol(0)>,<CustomCondition:Symbol(456):"bar":Symbol(0)>}',
          );
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
            [
              createCustomCondition(createSymbol(456), createString('bar'), createSymbol(0)),
              createInt(4),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), `${3 + 4}`);
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
            [
              createCustomCondition(createSymbol(456), createString('bar'), createSymbol(0)),
              createInt(4),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), `${3 + 4}`);
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
        {
          const state = createHashmap([
            [
              createCustomCondition(createSymbol(123), createString('foo'), createSymbol(0)),
              createInt(3),
            ],
            [
              createCustomCondition(createSymbol(456), createString('bar'), createSymbol(0)),
              createInt(4),
            ],
            [
              createCustomCondition(createSymbol(789), createString('baz'), createSymbol(0)),
              createInt(5),
            ],
          ]);
          const [result, dependencies] = evaluate(expression, state);
          assert.strictEqual(format(result), `${3 + 4}`);
          assert.deepEqual(
            getStateDependencies(dependencies).map((dependency) => format(dependency)),
            [
              '<CustomCondition:Symbol(456):"bar":Symbol(0)>',
              '<CustomCondition:Symbol(123):"foo":Symbol(0)>',
            ],
          );
        }
      })();
    });
  });
};
