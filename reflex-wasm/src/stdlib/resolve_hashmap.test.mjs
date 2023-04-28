// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveHashmap', (test) => {
    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createHashmap,
      createLambda,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getHashmapNumEntries,
      getHashmapValue,
      isHashmap,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashmap),
          createUnitList(createHashmap([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashmap),
          createUnitList(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(getHashmapValue(result, createString('qux'))), 'NULL');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashmap),
          createUnitList(
            createHashmap([
              [
                createApplication(createLambda(0, createString('foo')), createEmptyList()),
                createApplication(createLambda(0, createInt(3)), createEmptyList()),
              ],
              [
                createApplication(createLambda(0, createString('bar')), createEmptyList()),
                createApplication(createLambda(0, createInt(4)), createEmptyList()),
              ],
              [
                createApplication(createLambda(0, createString('baz')), createEmptyList()),
                createApplication(createLambda(0, createInt(5)), createEmptyList()),
              ],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(getHashmapValue(result, createString('qux'))), 'NULL');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveHashmap),
          createUnitList(
            createHashmap([
              [
                createApplication(createLambda(0, createString('foo')), createEmptyList()),
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createLambda(0, createInt(1)), createEmptyList()),
                      createApplication(createLambda(0, createInt(2)), createEmptyList()),
                      createApplication(createLambda(0, createInt(3)), createEmptyList()),
                    ),
                  ),
                  createEmptyList(),
                ),
              ],
              [
                createApplication(createLambda(0, createString('bar')), createEmptyList()),
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createLambda(0, createInt(4)), createEmptyList()),
                      createApplication(createLambda(0, createInt(5)), createEmptyList()),
                      createApplication(createLambda(0, createInt(6)), createEmptyList()),
                    ),
                  ),
                  createEmptyList(),
                ),
              ],
              [
                createApplication(createLambda(0, createString('baz')), createEmptyList()),
                createApplication(
                  createLambda(
                    0,
                    createTriple(
                      createApplication(createLambda(0, createInt(7)), createEmptyList()),
                      createApplication(createLambda(0, createInt(8)), createEmptyList()),
                      createApplication(createLambda(0, createInt(9)), createEmptyList()),
                    ),
                  ),
                  createEmptyList(),
                ),
              ],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(
          format(getHashmapValue(result, createString('foo'))),
          '[(0) => 1(), (0) => 2(), (0) => 3()]',
        );
        assert.strictEqual(
          format(getHashmapValue(result, createString('bar'))),
          '[(0) => 4(), (0) => 5(), (0) => 6()]',
        );
        assert.strictEqual(
          format(getHashmapValue(result, createString('baz'))),
          '[(0) => 7(), (0) => 8(), (0) => 9()]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
