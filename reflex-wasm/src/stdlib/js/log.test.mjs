// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Log', (test) => {
    test('(List)', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
      createInt,
      createString,
      createSignal,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Log),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Log),
          createUnitList(
            createTriple(
              createInt(3),
              createInt(4),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(createSignal(createErrorCondition(createString('foo')))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createErrorCondition,
      createInt,
      createMapIterator,
      createString,
      createSignal,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Log),
          createUnitList(
            createMapIterator(
              createTriple(
                createInt(3),
                createInt(4),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createSignal(createErrorCondition(createString('foo')))),
                ),
              ),
              createBuiltin(Stdlib.Identity),
            ),
          ),
        );
        debugger;
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
