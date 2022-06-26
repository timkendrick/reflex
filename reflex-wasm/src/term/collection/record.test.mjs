// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('TermType::Record', (test) => {
    test.skip('format', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('hash', (assert) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });

    test('toJson', (assert, {
      createEmptyList,
      createUnitList,
      createRecord,
      createString,
      createTriple,
      createInt,
      getStringValue,
      toJson,
    }) => {
      assert.strictEqual(
        getStringValue(toJson(createRecord(createEmptyList(), createEmptyList()))),
        '{}',
      );
      assert.strictEqual(
        getStringValue(
          toJson(createRecord(createUnitList(createString('foo')), createUnitList(createInt(3)))),
        ),
        '{"foo":3}',
      );
      assert.strictEqual(
        getStringValue(
          toJson(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        ),
        '{"foo":3,"bar":4,"baz":5}',
      );
      assert.strictEqual(
        getStringValue(
          toJson(
            createRecord(
              createTriple(createString('foo'), createInt(0), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        ),
        '{"foo":3,"baz":5}',
      );
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

    test('[simple] invalid keys', (assert, {
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
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(record, createString('invalid')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{(<InvalidAccessor:{ "foo": 3, "bar": 4, "baz": 5 },"invalid"> . NULL)}',
      );
      assert.strictEqual(format(dependencies), 'NULL');
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
        createApplication(createBuiltin(Stdlib.Get), createPair(record, createString(`key:${index}`))),
      );
      const results = expressions.map((expression) => evaluate(expression, NULL));
      results.forEach(([result, dependencies], index) => {
        assert.strictEqual(result, entries[index][1]);
        assert.strictEqual(format(dependencies), 'NULL');
      });
    });

    test('[hashmap] invalid keys', (assert, {
      createApplication,
      asSignal,
      createBuiltin,
      createInt,
      createList,
      createPair,
      createRecord,
      createString,
      evaluate,
      format,
      getInvalidAccessorConditionTarget,
      getInvalidAccessorConditionKey,
      getConditionType,
      getTreeLeft,
      getTreeLength,
      getSignalConditions,
      NULL,
      ConditionType,
      Stdlib,
    }) => {
      const entries = Array.from({ length: 128 }).map((_, index) => [
        createString(`key:${index}`),
        createInt(index),
      ]);
      const keys = createList(entries.map(([key, _]) => key));
      const values = createList(entries.map(([_, value]) => value));
      const record = createRecord(keys, values);
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(record, createString('foo')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      const signal = asSignal(result);
      assert.ok(signal);
      const conditions = getSignalConditions(signal);
      assert.strictEqual(getTreeLength(conditions), 1);
      const error = getTreeLeft(conditions);
      assert.strictEqual(getConditionType(error), ConditionType.InvalidAccessor);
      assert.strictEqual(getInvalidAccessorConditionTarget(error), record);
      assert.strictEqual(format(getInvalidAccessorConditionKey(error)), '"foo"');
      assert.strictEqual(format(dependencies), 'NULL');
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
          createBuiltin(Stdlib.CollectList),
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
          createBuiltin(Stdlib.CollectList),
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
