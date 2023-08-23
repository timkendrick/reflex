// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveHashset', (test) => {
    test('(Hashset)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createHashset,
      createLambda,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      getStateDependencies,
      getHashsetNumEntries,
      hasHashsetValue,
      isHashset,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashset),
          createUnitList(createHashset([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 0);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashset),
          createUnitList(
            createHashset([createString('foo'), createString('bar'), createString('baz')]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashset),
          createUnitList(
            createHashset([
              createApplication(createLambda(0, createString('foo')), createEmptyList()),
              createApplication(createLambda(0, createString('bar')), createEmptyList()),
              createApplication(createLambda(0, createString('baz')), createEmptyList()),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashset),
          createUnitList(
            createHashset([
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
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(
          hasHashsetValue(
            result,
            createTriple(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
            ),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createTriple(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
            ),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createTriple(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
            ),
          ),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(result, createTriple(createInt(1), createInt(2), createInt(3))),
          false,
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
