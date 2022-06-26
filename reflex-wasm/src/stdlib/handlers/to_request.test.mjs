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
        '{ "url": "http://example.com/", "method": "GET", "headers": {}, "body": null }',
      );
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createList,
      createRecord,
      createString,
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
              ]),
              createList([
                createString('http://example.com/'),
                createString('POST'),
                createRecord(
                  createUnitList(createString('foo')),
                  createUnitList(createString('bar')),
                ),
                createString('baz'),
              ]),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "url": "http://example.com/", "method": "POST", "headers": { "foo": "bar" }, "body": "baz" }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ToRequest),
          createUnitList(
            createRecord(
              createList([
                createString('body'),
                createString('headers'),
                createString('method'),
                createString('url'),
              ]),
              createList([
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
          '{ "url": "http://example.com/", "method": "POST", "headers": { "foo": "bar" }, "body": "baz" }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};
