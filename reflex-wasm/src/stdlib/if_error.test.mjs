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
      getStateDependencies,
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
        assert.deepEqual(getStateDependencies(dependencies), []);
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
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
            createBuiltin(Stdlib.Identity),
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
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
          ]),
        );
        assert.strictEqual(format(result), '["foo"]');
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
        );
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.IfError),
          createPair(
            createEffect(createCustomCondition(createSymbol(123), createInt(3), createSymbol(0))),
            createBuiltin(Stdlib.Length),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
          ]),
        );
        assert.strictEqual(format(result), '1');
        assert.deepEqual(
          getStateDependencies(dependencies).map((dependency) => format(dependency)),
          ['<CustomCondition:Symbol(123):3:Symbol(0)>'],
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
            createBuiltin(Stdlib.Identity),
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
          createBuiltin(Stdlib.IfError),
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
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)), createInt(3)],
            [createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)), createInt(4)],
            [createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)), createInt(5)],
            [createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)), createInt(6)],
          ]),
        );
        assert.strictEqual(format(result), `${3 + 4 + 5 + 6}`);
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
          createBuiltin(Stdlib.IfError),
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
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
              createSignal(createErrorCondition(createString('bar'))),
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
          createBuiltin(Stdlib.IfError),
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
            createBuiltin(Stdlib.Identity),
          ),
        );
        const [result, dependencies] = evaluate(
          expression,
          createHashmap([
            [
              createCustomCondition(createSymbol(123), createInt(3), createSymbol(0)),
              createSignal(createErrorCondition(createString('foo'))),
            ],
            [
              createCustomCondition(createSymbol(234), createInt(4), createSymbol(0)),
              createSignal(createErrorCondition(createString('bar'))),
            ],
            [
              createCustomCondition(createSymbol(345), createInt(5), createSymbol(0)),
              createSignal(createErrorCondition(createString('baz'))),
            ],
            [
              createCustomCondition(createSymbol(456), createInt(6), createSymbol(0)),
              createSignal(createErrorCondition(createString('qux'))),
            ],
          ]),
        );
        assert.strictEqual(format(result), '["foo", "bar", "baz", "qux"]');
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
