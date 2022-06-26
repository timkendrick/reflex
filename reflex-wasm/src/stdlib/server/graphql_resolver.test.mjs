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
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
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
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(1) => { "query": null, "mutation": null, "subscription": null }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
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
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(1) => { "query": "foo", "mutation": "bar", "subscription": "baz" }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
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
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(1) => { "subscription": "foo", "query": "bar", "mutation": "baz" }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
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
          '{(<InvalidFunctionArgs:GraphQlResolver({ "query": "foo", "mutation": "bar" })> . NULL)}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Lambda)', (assert, {
      createApplication,
      createRecord,
      createBuiltin,
      createLambda,
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
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(1) => (0) => { "query": "foo", "mutation": "bar", "subscription": "baz" }()',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
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
                createTriple(createString('foo'), createString('bar'), createString('baz')),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '(1) => { "query": "foo", "mutation": "bar", "subscription": "baz" }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
