// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Set', (test) => {
    test('(List, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(0),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[6, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(1),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 6, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createInt(2),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List, Float, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createFloat,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createFloat(0.0),
            createInt(3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createFloat(0.0),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[6, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createFloat(1.0),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 6, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createFloat(2.0),
            createInt(6),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record, String, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createRecord,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(record, createString('qux'), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5, "qux": 6 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(record, createString('bar'), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(record, createString('bar'), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 6, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashmap, String, Int)', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      getHashmapValue,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const hashmap = createHashmap([
          [createString('foo'), createInt(3)],
          [createString('bar'), createInt(4)],
          [createString('baz'), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(hashmap, createString('qux'), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(4)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(getHashmapValue(result, createString('qux'))), '6');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashmap([
          [createString('foo'), createInt(3)],
          [createString('bar'), createInt(4)],
          [createString('baz'), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(hashmap, createString('bar'), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(3)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashmap([
          [createString('foo'), createInt(3)],
          [createString('bar'), createInt(4)],
          [createString('baz'), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Set),
          createTriple(hashmap, createString('bar'), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'Map(3)');
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '6');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
