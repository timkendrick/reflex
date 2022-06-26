// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_IfError', (test) => {
    test('(Int, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(createInt(3), createBuiltin(Stdlib.Identity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Effect, Builtin)', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createEffect,
      createErrorCondition,
      createHashmap,
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
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3))),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3))),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
          ]),
        );
        assert.strictEqual(format(result), '[<Error:"foo">]');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3))),
            createBuiltin(Stdlib.Length),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
          ]),
        );
        assert.strictEqual(format(result), '1');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Add),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(123), createInt(3))),
                    createEffect(createCustomCondition(createSymbol(234), createInt(4))),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(345), createInt(5))),
                    createEffect(createCustomCondition(createSymbol(456), createInt(6))),
                  ),
                ),
              ),
            ),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{(((<Custom:Symbol(123):3> . NULL) . (<Custom:Symbol(234):4> . NULL)) . ((<Custom:Symbol(345):5> . NULL) . (<Custom:Symbol(456):6> . NULL)))}',
        );
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . ((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL)))',
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Add),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(123), createInt(3))),
                    createEffect(createCustomCondition(createSymbol(234), createInt(4))),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(345), createInt(5))),
                    createEffect(createCustomCondition(createSymbol(456), createInt(6))),
                  ),
                ),
              ),
            ),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3)), createInt(3)],
            [createCustomCondition(createSymbol(234), createInt(4)), createInt(4)],
            [createCustomCondition(createSymbol(345), createInt(5)), createInt(5)],
            [createCustomCondition(createSymbol(456), createInt(6)), createInt(6)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4 + 5 + 6}`);
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . ((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL)))',
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Add),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(123), createInt(3))),
                    createEffect(createCustomCondition(createSymbol(234), createInt(4))),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(345), createInt(5))),
                    createEffect(createCustomCondition(createSymbol(456), createInt(6))),
                  ),
                ),
              ),
            ),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(234), createInt(4)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6)),
              createSignal(createErrorCondition(createString('bar'))),
            ],
          ]),
        );
        assert.strictEqual(
          format(result),
          '{(<Custom:Symbol(123):3> . (<Custom:Symbol(345):5> . NULL))}',
        );
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . ((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL)))',
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Add),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(123), createInt(3))),
                    createEffect(createCustomCondition(createSymbol(234), createInt(4))),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(createCustomCondition(createSymbol(345), createInt(5))),
                    createEffect(createCustomCondition(createSymbol(456), createInt(6))),
                  ),
                ),
              ),
            ),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
            [
              createCustomCondition(createSymbol(234), createInt(4)),
              createSignal(createErrorCondition(createString('bar'))),
            ],
            [
              createCustomCondition(createSymbol(345), createInt(5)),
              createSignal(createErrorCondition(createString('baz'))),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6)),
              createSignal(createErrorCondition(createString('qux'))),
            ],
          ]),
        );
        assert.strictEqual(
          format(result),
          '[<Error:"foo">, <Error:"bar">, <Error:"baz">, <Error:"qux">]',
        );
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . ((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL)))',
        );
      })();
    });
  });
};
