// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ToRequest', (test) => {
    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.ToRequest),
        createUnitList(createString('http://example.com/')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        '{ "url": "http://example.com/", "method": "GET", "headers": {}, "body": null, "token": null }',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createList,
      createRecord,
      createString,
      createSymbol,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ToRequest),
          createUnitList(
            createRecord(
              createList([
                createString('url'),
                createString('method'),
                createString('headers'),
                createString('body'),
                createString('token'),
              ]),
              createList([
                createString('http://example.com/'),
                createString('POST'),
                createRecord(
                  createUnitList(createString('foo')),
                  createUnitList(createString('bar')),
                ),
                createString('baz'),
                createSymbol(123),
              ]),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "url": "http://example.com/", "method": "POST", "headers": { "foo": "bar" }, "body": "baz", "token": Symbol(123) }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ToRequest),
          createUnitList(
            createRecord(
              createList([
                createString('token'),
                createString('body'),
                createString('headers'),
                createString('method'),
                createString('url'),
              ]),
              createList([
                createSymbol(123),
                createString('baz'),
                createRecord(
                  createUnitList(createString('foo')),
                  createUnitList(createString('bar')),
                ),
                createString('POST'),
                createString('http://example.com/'),
              ]),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "url": "http://example.com/", "method": "POST", "headers": { "foo": "bar" }, "body": "baz", "token": Symbol(123) }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
