// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Merge', (test) => {
    test('(...)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createString,
      createRangeIterator,
      createRecord,
      createTriple,
      createZipIterator,
      evaluate,
      format,
      getStateDependencies,
      isRecord,
      getListItems,
      getListLength,
      getRecordKeys,
      getRecordValues,
      NULL,
      TRUE,
      Stdlib,
    }) => {
      function formatSortedRecord(result) {
        const keys = getListItems(getRecordKeys(result));
        const values = getListItems(getRecordValues(result));
        const entries = keys.map((key, index) => [key, values[index]]);
        return entries.length == 0
          ? '{}'
          : `{ ${entries
              .map(([key, value]) => [key, value, format(key), format(value)])
              .sort((a, b) => (a[2] > b[2] ? 1 : -1))
              .map(
                ([_key, _value, formattedKey, formattedValue]) =>
                  `${formattedKey}: ${formattedValue}`,
              )
              .join(', ')} }`;
      }

      (() => {
        const expression = createApplication(createBuiltin(Stdlib.Merge), createEmptyList());
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Merge),
          createTriple(
            createZipIterator(
              createTriple(createString('a'), createString('b'), createString('c')),
              createRangeIterator(3, 3),
            ),
            createZipIterator(
              createTriple(createString('d'), createString('e'), createString('f')),
              createRangeIterator(6, 3),
            ),
            createZipIterator(
              createTriple(createString('g'), createString('h'), createString('i')),
              createRangeIterator(9, 3),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        const expected = { a: 3, b: 4, c: 5, d: 6, e: 7, f: 8, g: 9, h: 10, i: 11 };
        assert.strictEqual(isRecord(result), TRUE);
        assert.strictEqual(getListLength(getRecordKeys(result)), Object.keys(expected).length);
        assert.strictEqual(getListLength(getRecordValues(result)), Object.values(expected).length);
        assert.strictEqual(
          formatSortedRecord(result),
          '{ "a": 3, "b": 4, "c": 5, "d": 6, "e": 7, "f": 8, "g": 9, "h": 10, "i": 11 }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Merge),
          createTriple(
            createRecord(
              createTriple(createString('a'), createString('b'), createString('c')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createRecord(
              createTriple(createString('d'), createString('e'), createString('f')),
              createTriple(createInt(6), createInt(7), createInt(8)),
            ),
            createRecord(
              createTriple(createString('g'), createString('h'), createString('i')),
              createTriple(createInt(9), createInt(10), createInt(11)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          formatSortedRecord(result),
          '{ "a": 3, "b": 4, "c": 5, "d": 6, "e": 7, "f": 8, "g": 9, "h": 10, "i": 11 }',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};
