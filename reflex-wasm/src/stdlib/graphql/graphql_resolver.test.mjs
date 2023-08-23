// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_GraphQlResolver', (test) => {
    test('(Record)', (assert, {
      createApplication,
      createRecord,
      createBuiltin,
      createNil,
      createPair,
      createString,
      createSymbol,
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
          createApplication(
            createBuiltin(Stdlib.GraphQlResolver),
            createUnitList(
              createRecord(
                createTriple(
                  createString('query'),
                  createString('mutation'),
                  createString('subscription'),
                ),
                createTriple(createNil(), createNil(), createNil()),
              ),
            ),
          ),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "query": null, "mutation": null, "subscription": null }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.GraphQlResolver),
            createUnitList(
              createRecord(
                createTriple(
                  createString('query'),
                  createString('mutation'),
                  createString('subscription'),
                ),
                createTriple(createString('foo'), createString('bar'), createString('baz')),
              ),
            ),
          ),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "query": "foo", "mutation": "bar", "subscription": "baz" }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.GraphQlResolver),
            createUnitList(
              createRecord(
                createTriple(
                  createString('subscription'),
                  createString('query'),
                  createString('mutation'),
                ),
                createTriple(createString('foo'), createString('bar'), createString('baz')),
              ),
            ),
          ),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "subscription": "foo", "query": "bar", "mutation": "baz" }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.GraphQlResolver),
          createUnitList(
            createRecord(
              createPair(createString('query'), createString('mutation')),
              createPair(createString('foo'), createString('bar')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:GraphQlResolver({ "query": "foo", "mutation": "bar" })>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Lambda)', (assert, {
      createApplication,
      createRecord,
      createBuiltin,
      createLambda,
      createString,
      createSymbol,
      createTriple,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.GraphQlResolver),
            createUnitList(
              createLambda(
                0,
                createRecord(
                  createTriple(
                    createString('query'),
                    createString('mutation'),
                    createString('subscription'),
                  ),
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                ),
              ),
            ),
          ),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "query": "foo", "mutation": "bar", "subscription": "baz" }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.GraphQlResolver),
            createUnitList(
              createLambda(
                1,
                createRecord(
                  createTriple(
                    createString('query'),
                    createString('mutation'),
                    createString('subscription'),
                  ),
                  createTriple(createString('foo'), createString('bar'), createVariable(0)),
                ),
              ),
            ),
          ),
          createUnitList(createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "query": "foo", "mutation": "bar", "subscription": Symbol(123) }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
