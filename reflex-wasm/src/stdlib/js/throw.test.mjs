// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Throw', (test) => {
    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.Throw),
        createUnitList(createString('foo')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{<ErrorCondition:"foo">}');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createPair,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Throw),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('message')),
              createPair(createString('Error'), createString('foo')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:{ "name": "Error", "message": "foo" }>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Throw),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('errors')),
              createPair(createString('AggregateError'), createEmptyList()),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:{ "name": "AggregateError", "errors": [] }>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Throw),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('errors')),
              createPair(
                createString('AggregateError'),
                createUnitList(
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString('foo')),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:{ "name": "Error", "message": "foo" }>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Throw),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('errors')),
              createPair(
                createString('AggregateError'),
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
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:{ "name": "Error", "message": "foo" }>,<ErrorCondition:{ "name": "Error", "message": "bar" }>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Throw),
          createUnitList(
            createRecord(
              createPair(createString('name'), createString('errors')),
              createPair(
                createString('AggregateError'),
                createTriple(
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString('foo')),
                  ),
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString('bar')),
                  ),
                  createRecord(
                    createPair(createString('name'), createString('message')),
                    createPair(createString('Error'), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:{ "name": "Error", "message": "foo" }>,<ErrorCondition:{ "name": "Error", "message": "bar" }>,<ErrorCondition:{ "name": "Error", "message": "baz" }>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
