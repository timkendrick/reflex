// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_IfPending', (test) => {
    test('(Int, Int)', (assert, {
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
          createBuiltin(Stdlib.IfPending),
          createPair(createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Effect, Int)', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createEffect,
      createHashmap,
      createInt,
      createPair,
      createPendingCondition,
      createSignal,
      createSymbol,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3))),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<Custom:Symbol(123):3> . NULL)}');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3))),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3)),
              createSignal(createPendingCondition()),
            ],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), '(<Custom:Symbol(123):3> . NULL)');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
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
            createInt(3),
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
          createBuiltin(Stdlib.IfPending),
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
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(234), createInt(4)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6)),
              createSignal(createPendingCondition()),
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
          createBuiltin(Stdlib.IfPending),
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
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(234), createInt(4)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(345), createInt(5)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6)),
              createSignal(createPendingCondition()),
            ],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.strictEqual(
          format(dependencies),
          '(((<Custom:Symbol(456):6> . NULL) . (<Custom:Symbol(345):5> . NULL)) . ((<Custom:Symbol(234):4> . NULL) . (<Custom:Symbol(123):3> . NULL)))',
        );
      })();
    });
  });
};
