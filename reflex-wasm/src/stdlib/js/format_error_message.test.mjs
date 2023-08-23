// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_FormatErrorMessage', (test) => {
    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'foo');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(createString('"foo"')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '"foo"');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createRecord(createPair(createString('message')), createPair(createString('foo'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'foo');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('message')),
              createPair(createString('Error'), createString('foo')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Error: foo');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('message')),
              createPair(createString('Error'), createInt(3)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Error: 3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createList,
      createPair,
      createRecord,
      createString,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), '');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createUnitList(
              createRecord(
                createPair(createString('name'), createString('message')),
                createPair(createString('Error'), createString('foo')),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Error: foo');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createPair(
              createRecord(
                createPair(createString('name'), createString('message')),
                createPair(createString('Error'), createString('foo')),
              ),
              createRecord(
                createPair(createString('name'), createString('message')),
                createPair(createString('Error'), createString('bar')),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(getStringValue(result), 'Error: foo\nError: bar');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createList(
              Array.from({ length: 9 }, (_, i) =>
                createRecord(
                  createPair(createString('name'), createString('message')),
                  createPair(createString('Error'), createString(`Item ${i + 1}`)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createList(
              Array.from({ length: 10 }, (_, i) =>
                createRecord(
                  createPair(createString('name'), createString('message')),
                  createPair(createString('Error'), createString(`Item ${i + 1}`)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9\nError: Item 10',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createList(
              Array.from({ length: 11 }, (_, i) =>
                createRecord(
                  createPair(createString('name'), createString('message')),
                  createPair(createString('Error'), createString(`Item ${i + 1}`)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9\n...2 more errors',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createList(
              Array.from({ length: 12 }, (_, i) =>
                createRecord(
                  createPair(createString('name'), createString('message')),
                  createPair(createString('Error'), createString(`Item ${i + 1}`)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9\n...3 more errors',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createFilterIterator,
      createLambda,
      createMapIterator,
      createList,
      createPair,
      createRecord,
      createString,
      createUnitList,
      evaluate,
      getStateDependencies,
      getStringValue,
      isString,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createMapIterator(
              createList(
                Array.from({ length: 12 }, (_, i) =>
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString(`Item ${i + 1}`)),
                  ),
                ),
              ),
              createBuiltin(Stdlib.Identity),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9\n...3 more errors',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.FormatErrorMessage),
          createUnitList(
            createFilterIterator(
              createList(
                Array.from({ length: 12 }, (_, i) =>
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString(`Item ${i + 1}`)),
                  ),
                ),
              ),
              createLambda(1, createBoolean(true)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isString(result));
        assert.strictEqual(
          getStringValue(result),
          'Error: Item 1\nError: Item 2\nError: Item 3\nError: Item 4\nError: Item 5\nError: Item 6\nError: Item 7\nError: Item 8\nError: Item 9\n...3 more errors',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
