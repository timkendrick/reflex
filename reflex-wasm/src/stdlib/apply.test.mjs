// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Apply', (test) => {
    test('(Builtin, List)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createPair,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Add), createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Add()> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Add), createPair(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Builtin, Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createInt,
      createOnceIterator,
      createRangeIterator,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Add), createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{(<InvalidFunctionArgs:Add()> . NULL)}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Abs), createOnceIterator(createInt(-3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(createBuiltin(Stdlib.Add), createRangeIterator(3, 2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 + 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
