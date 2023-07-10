export default (describe) => {
  describe('Stdlib_ResolveLoaderResults', (test) => {
    test('List results', (assert, {
      createApplication,
      createBuiltin,
      createList,
      createPair,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createTriple(
              createString('value:foo'),
              createString('value:bar'),
              createString('value:baz'),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["value:foo", "value:bar", "value:baz"]');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createPair(createString('value:foo'), createString('value:bar')),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{"Expected 3 results, received 2"}');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createList([
              createString('value:foo'),
              createString('value:bar'),
              createString('value:baz'),
              createString('value:qux'),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{"Expected 3 results, received 4"}');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });

    test('Hashmap results', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createPair,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createHashmap([
              [createString('foo'), createString('value:foo')],
              [createString('bar'), createString('value:bar')],
              [createString('baz'), createString('value:baz')],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["value:foo", "value:bar", "value:baz"]');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createHashmap([
              [createString('foo'), createString('value:foo')],
              [createString('bar'), createString('value:bar')],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"Missing result for key: baz"');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createHashmap([
              [createString('foo'), createString('value:foo')],
              [createString('bar'), createString('value:bar')],
              [createString('baz'), createString('value:baz')],
              [createString('qux'), createString('value:qux')],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"Unexpected key: qux"');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });

    test('Record results', (assert, {
      createApplication,
      createBuiltin,
      createList,
      createPair,
      createRecord,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(
                createString('value:foo'),
                createString('value:bar'),
                createString('value:baz'),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["value:foo", "value:bar", "value:baz"]');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createRecord(
              createTriple(createString('foo'), createString('bar')),
              createTriple(createString('value:foo'), createString('value:bar')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"Missing result for key: baz"');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createRecord(
              createList([
                createString('foo'),
                createString('bar'),
                createString('baz'),
                createString('qux'),
              ]),
              createList([
                createString('value:foo'),
                createString('value:bar'),
                createString('value:baz'),
                createString('value:qux'),
              ]),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"Unexpected key: qux"');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });

    test('Iterator results', (assert, {
      createApplication,
      createBuiltin,
      createLambda,
      createList,
      createMapIterator,
      createPair,
      createString,
      createTriple,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createMapIterator(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.CollectString),
                  createPair(createString('value:'), createVariable(0)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["value:foo", "value:bar", "value:baz"]');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createMapIterator(
              createPair(createString('foo'), createString('bar')),
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.CollectString),
                  createPair(createString('value:'), createVariable(0)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{"Expected 3 results, received 2"}');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveLoaderResults),
          createPair(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
            createMapIterator(
              createList([
                createString('foo'),
                createString('bar'),
                createString('baz'),
                createString('qux'),
              ]),
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.CollectString),
                  createPair(createString('value:'), createVariable(0)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{"Expected 3 results, received 4"}');
        assert.deepEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
