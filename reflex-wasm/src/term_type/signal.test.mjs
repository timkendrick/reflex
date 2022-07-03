// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Signal', (test) => {
    test('format', (assert, {
      createErrorCondition,
      createSignal,
      createString,
      createTree,
      format,
      NULL,
    }) => {
      assert.strictEqual(
        format(createSignal(createTree(createErrorCondition(createString('foo')), NULL))),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(createSignal(createTree(NULL, createErrorCondition(createString('foo'))))),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createErrorCondition(createString('foo')),
              createErrorCondition(createString('bar')),
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(createTree(createErrorCondition(createString('foo')), NULL), NULL),
          ),
        ),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(createTree(NULL, createErrorCondition(createString('foo'))), NULL),
          ),
        ),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(NULL, createTree(createErrorCondition(createString('foo')), NULL)),
          ),
        ),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(NULL, createTree(NULL, createErrorCondition(createString('foo')))),
          ),
        ),
        '{<ErrorCondition:"foo">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createTree(createErrorCondition(createString('foo')), NULL),
              createTree(createErrorCondition(createString('bar')), NULL),
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createTree(
                createErrorCondition(createString('foo')),
                createErrorCondition(createString('bar')),
              ),
              createTree(createErrorCondition(createString('baz')), NULL),
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createTree(createErrorCondition(createString('foo')), NULL),
              createTree(
                createErrorCondition(createString('bar')),
                createErrorCondition(createString('baz')),
              ),
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createTree(
                createErrorCondition(createString('foo')),
                createTree(
                  createErrorCondition(createString('bar')),
                  createErrorCondition(createString('baz')),
                ),
              ),
              NULL,
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              NULL,
              createTree(
                createErrorCondition(createString('foo')),
                createTree(
                  createErrorCondition(createString('bar')),
                  createErrorCondition(createString('baz')),
                ),
              ),
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
      );
      assert.strictEqual(
        format(
          createSignal(
            createTree(
              createTree(
                createErrorCondition(createString('foo')),
                createTree(
                  createErrorCondition(createString('bar')),
                  createTree(createErrorCondition(createString('baz')), NULL),
                ),
              ),
              NULL,
            ),
          ),
        ),
        '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
      );
    });

    test.skip('hash', (assert, {}) => {
      throw new Error('Not yet implemented');
    });

    test.skip('equals', (assert) => {
      throw new Error('Not yet implemented');
    });
  });
};
