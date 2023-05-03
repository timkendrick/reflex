// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveRecord', (test) => {
    test('(Record)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createLambda,
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
          createBuiltin(Stdlib.ResolveRecord),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveRecord),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveRecord),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveRecord),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                    ),
                  ),
                  createEmptyList(),
                ),
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
                    ),
                  ),
                  createEmptyList(),
                ),
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
                    ),
                  ),
                  createEmptyList(),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "foo": [Abs(-1), Abs(-2), Abs(-3)], "bar": [Abs(-4), Abs(-5), Abs(-6)], "baz": [Abs(-7), Abs(-8), Abs(-9)] }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
