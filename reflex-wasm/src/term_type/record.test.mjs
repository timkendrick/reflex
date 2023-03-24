// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Record', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });

    test('[simple] basic property access', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createRecord,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const prototype = createTriple(createString('foo'), createString('bar'), createString('baz'));
      const record = createRecord(
        prototype,
        createTriple(createInt(3), createInt(4), createInt(5)),
      );
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString('baz')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('[hashmap] basic property access', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createList,
      createPair,
      createRecord,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const entries = Array.from({ length: 128 }).map((_, index) => [
        createString(`key:${index}`),
        createInt(index),
      ]);
      const keys = createList(entries.map(([key, _]) => key));
      const values = createList(entries.map(([_, value]) => value));
      const record = createRecord(keys, values);
      const expressions = entries.map((_, index) =>
        createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString(`key:${index}`)),
        ),
      );
      const results = expressions.map((expression) => evaluate(expression, NULL));
      results.forEach(([result, dependencies], index) => {
        assert.strictEqual(result, entries[index][1]);
        assert.strictEqual(format(dependencies), 'NULL');
      });
    });

    test('iteration', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createRecord(createEmptyList(), createEmptyList())),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(3), createInt(4), createInt(5)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[["foo", 3], ["bar", 4], ["baz", 5]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
