// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_IfPending', (test) => {
    test('(Int, Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
      createPair,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
          createPair(createInt(3), createLambda(0, createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Effect, Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createCustomCondition,
      createEffect,
      createHashmap,
      createInt,
      createLambda,
      createPair,
      createPendingCondition,
      createSignal,
      createSymbol,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
            createLambda(0, createInt(3)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<CustomCondition:Symbol(123):3:Symbol(0)>}');
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfPending),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
            createLambda(0, createInt(3)),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
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
                    createEffect(
                      createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
                    ),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(
                      createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
                    ),
                  ),
                ),
              ),
            ),
            createLambda(0, createInt(3)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<CustomCondition:Symbol(123):3:Symbol(0)>,<CustomCondition:Symbol(234):4:Symbol(0)>,<CustomCondition:Symbol(345):5:Symbol(0)>,<CustomCondition:Symbol(456):6:Symbol(0)>}',
        );
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(456):6:Symbol(0)>',
            '<CustomCondition:Symbol(345):5:Symbol(0)>',
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
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
                    createEffect(
                      createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
                    ),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(
                      createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
                    ),
                  ),
                ),
              ),
            ),
            createLambda(0, createInt(3)),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
          ]),
        );
        assert.strictEqual(
          format(result),
          '{<CustomCondition:Symbol(123):3:Symbol(0)>,<CustomCondition:Symbol(345):5:Symbol(0)>}',
        );
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(456):6:Symbol(0)>',
            '<CustomCondition:Symbol(345):5:Symbol(0)>',
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
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
                    createEffect(
                      createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
                    ),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Add),
                  createPair(
                    createEffect(
                      createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
                    ),
                    createEffect(
                      createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
                    ),
                  ),
                ),
              ),
            ),
            createLambda(0, createInt(3)),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
              createSignal(createPendingCondition()),
            ],
          ]),
        );
        assert.strictEqual(format(result), '3');
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          [
            '<CustomCondition:Symbol(456):6:Symbol(0)>',
            '<CustomCondition:Symbol(345):5:Symbol(0)>',
            '<CustomCondition:Symbol(234):4:Symbol(0)>',
            '<CustomCondition:Symbol(123):3:Symbol(0)>',
          ],
        );
      })();
    });
  });
};
