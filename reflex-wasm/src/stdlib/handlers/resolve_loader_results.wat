(module
  (@const-string $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_MISSING_KEY "Missing result for key: ")
  (@const-string $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_UNEXPECTED_KEY "Unexpected key: ")
  (@const-string $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_INVALID_LENGTH_EXPECTED "Expected ")
  (@const-string $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_INVALID_LENGTH_RECEIVED " results, received ")

  (@builtin $Stdlib_ResolveLoaderResults "ResolveLoaderResults"
    (@args (@strict $keys) (@strict $results))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::List))
      (func $Stdlib_ResolveLoaderResults::impl::List::List (param $keys i32) (param $results i32) (param $state i32) (result i32 i32)
        ;; Ensure that the correct number of results have been returned for the requested key set
        (if
          (i32.ne
            (call $Term::List::get_length (local.get $results))
            (call $Term::List::get_length (local.get $keys)))
          (then
            (return
              (call $Term::Signal::of
                (call $Stdlib_ResolveLoaderResults::error::invalid_length
                  (call $Term::List::get_length (local.get $keys))
                  (call $Term::List::get_length (local.get $results))))
              (global.get $NULL))))
        ;; Return the list of results
        (local.get $results)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Hashmap))
      (func $Stdlib_ResolveLoaderResults::impl::List::Hashmap (param $keys i32) (param $results i32) (param $state i32) (result i32 i32)
        (local $num_keys i32)
        (local $values i32)
        (local $key i32)
        (local $value i32)
        (local $index i32)
        (local $keys_list i32)
        (local $expected_keys i32)
        ;; Copy the result values from the results hashmap into a list with the same order as the keys list
        (local.set $values
          (if (result i32)
            (local.tee $num_keys (call $Term::List::get_length (local.get $keys)))
            (then
              ;; Create a new list of the correct length to store the results
              (local.set $values (call $Term::List::allocate (local.get $num_keys)))
              ;; Copy each requested key's corresponding value from the results hashmap into the results list
              (loop $LOOP
                ;; Retrieve the value corresponding to the current key from the results hashmap
                (local.tee $value
                  (call $Term::Hashmap::traits::get
                    (local.get $results)
                    (local.tee $key (call $Term::List::get_item (local.get $keys) (local.get $index)))))
                ;; If the key was not present in the results hashmap, return an error result
                (if
                  (i32.eq (global.get $NULL))
                  (then
                    (return
                      (call $Stdlib_ResolveLoaderResults::error::missing_key (local.get $key))
                      (global.get $NULL))))
                ;; Set the corresponding item in the results list
                (call $Term::List::set_item (local.get $values) (local.get $index) (local.get $value))
                ;; If this was not the final key, continue with the next key
                (br_if $LOOP
                  (i32.lt_u
                    (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                    (local.get $num_keys))))
              ;; Now that all the values have been copied into the results list, instantiate the results list term
              (call $Term::List::init (local.get $values) (local.get $num_keys)))
            (else
              (call $Term::List::empty))))
        ;; If any unexpected results were provided in the results hashmap, return an error result
        (if
          (i32.ge_u
            (call $Term::Hashmap::get::num_entries (local.get $results))
            (local.get $num_keys))
          (then
            ;; Create a lookup hashmap to determine whether a provided key is in the set of expected keys
            (call $Term::Hashmap::traits::collect
              (call $Term::FlattenIterator::new
                (call $Term::ZipIterator::new (local.get $keys) (local.get $keys)))
              (global.get $NULL))
            (call $Dependencies::assert_empty)
            (local.set $expected_keys)
            ;; Collect a list of provided keys
            (call $Term::List::traits::collect
              (call $Term::HashmapKeysIterator::new (local.get $results))
              (global.get $NULL))
            (call $Dependencies::assert_empty)
            (local.set $keys_list)
            ;; Iterate over the received keys until an unexpected key is encountered
            (local.set $num_keys (call $Term::List::get_length (local.get $keys_list)))
            (local.set $index (i32.const 0))
            (loop $LOOP
              ;; If the key was not present in the results hashmap, return an error result
              (if
                (call $Utils::bool::not
                  (call $Term::Hashmap::traits::has
                    (local.get $expected_keys)
                    (local.tee $key (call $Term::List::get_item (local.get $keys_list) (local.get $index)))))
                (then
                  (call $Term::List::drop (local.get $keys_list))
                  (call $Term::Hashmap::drop (local.get $expected_keys))
                  (call $Term::List::drop (local.get $values))
                  (return
                    (call $Stdlib_ResolveLoaderResults::error::unexpected_key (local.get $key))
                    (global.get $NULL))))
              ;; If this was not the final key, continue with the next key
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $num_keys))))
            ;; Dispose the temporary lookup instances
            (call $Term::List::drop (local.get $keys_list))
            (call $Term::Hashmap::drop (local.get $expected_keys))))
        ;; Return the list of values
        (local.get $values)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Record))
      (func $Stdlib_ResolveLoaderResults::impl::List::Record (param $keys i32) (param $results i32) (param $state i32) (result i32 i32)
        (local $num_keys i32)
        (local $values i32)
        (local $key i32)
        (local $value i32)
        (local $index i32)
        (local $keys_list i32)
        (local $expected_keys i32)
        ;; Copy the result values from the results record into a list with the same order as the keys list
        (local.set $values
          (if (result i32)
            (local.tee $num_keys (call $Term::List::get_length (local.get $keys)))
            (then
              ;; Create a new list of the correct length to store the results
              (local.set $values (call $Term::List::allocate (local.get $num_keys)))
              ;; Copy each requested key's corresponding value from the results record into the results list
              (loop $LOOP
                ;; Retrieve the value corresponding to the current key from the results record
                (local.tee $value
                  (call $Term::Record::traits::get
                    (local.get $results)
                    (local.tee $key (call $Term::List::get_item (local.get $keys) (local.get $index)))))
                ;; If the key was not present in the results record, return an error result
                (if
                  (i32.eq (global.get $NULL))
                  (then
                    (return
                      (call $Stdlib_ResolveLoaderResults::error::missing_key (local.get $key))
                      (global.get $NULL))))
                ;; Set the corresponding item in the results list
                (call $Term::List::set_item (local.get $values) (local.get $index) (local.get $value))
                ;; If this was not the final key, continue with the next key
                (br_if $LOOP
                  (i32.lt_u
                    (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                    (local.get $num_keys))))
              ;; Now that all the values have been copied into the results list, instantiate the results list term
              (call $Term::List::init (local.get $values) (local.get $num_keys)))
            (else
              (call $Term::List::empty))))
        ;; If any unexpected results were provided in the results record, return an error result
        (if
          (i32.ge_u
            (call $Term::List::get_length (call $Term::Record::get::keys (local.get $results)))
            (local.get $num_keys))
          (then
            ;; Create a lookup record to determine whether a provided key is in the set of expected keys
            (call $Term::Hashmap::traits::collect
              (call $Term::FlattenIterator::new
                (call $Term::ZipIterator::new (local.get $keys) (local.get $keys)))
              (global.get $NULL))
            (call $Dependencies::assert_empty)
            (local.set $expected_keys)
            ;; Get the list of provided keys
            (local.set $keys_list (call $Term::Record::get::keys (local.get $results)))
            ;; Iterate over the received keys until an unexpected key is encountered
            (local.set $num_keys (call $Term::List::get_length (local.get $keys_list)))
            (local.set $index (i32.const 0))
            (loop $LOOP
              ;; If the key was not present in the results record, return an error result
              (if
                (call $Utils::bool::not
                  (call $Term::Hashmap::traits::has
                    (local.get $expected_keys)
                    (local.tee $key (call $Term::List::get_item (local.get $keys_list) (local.get $index)))))
                (then
                  (call $Term::List::drop (local.get $keys_list))
                  (call $Term::Hashmap::drop (local.get $expected_keys))
                  (call $Term::List::drop (local.get $values))
                  (return
                    (call $Stdlib_ResolveLoaderResults::error::unexpected_key (local.get $key))
                    (global.get $NULL))))
              ;; If this was not the final key, continue with the next key
              (br_if $LOOP
                (i32.lt_u
                  (local.tee $index (i32.add (local.get $index) (i32.const 1)))
                  (local.get $num_keys))))
            ;; Dispose the temporary lookup instances
            (call $Term::List::drop (local.get $keys_list))
            (call $Term::Hashmap::drop (local.get $expected_keys))))
        ;; Return the list of values
        (local.get $values)
        (global.get $NULL)))

    (@impl
      (i32.eq (global.get $TermType::List))
      (call $TermType::implements::iterate)
      (func $Stdlib_ResolveLoaderResults::impl::List::<iterate> (param $keys i32) (param $results i32) (param $state i32) (result i32 i32)
        (local $values i32)
        (local $dependencies i32)
        (call $Term::List::traits::collect_strict (local.get $results) (local.get $state))
        (local.set $dependencies)
        (local.tee $values)
        (if (result i32 i32)
          (call $Term::Signal::is)
          (then
            (local.get $values)
            (local.get $dependencies))
          (else
            (call $Stdlib_ResolveLoaderResults::impl::List::List (local.get $keys) (local.get $values) (local.get $state))
            (call $Dependencies::traits::union (local.get $dependencies))))))

    (@default
      (func $Stdlib_ResolveLoaderResults::default::impl (param $keys i32) (param $results i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_ResolveLoaderResults)
            (call $Term::List::create_pair (local.get $results) (local.get $keys))))
        (global.get $NULL))))

  (func $Stdlib_ResolveLoaderResults::error::missing_key (param $key i32) (result i32)
    ;; Generate a 'missing key: X' label string
    (call $Term::String::collect_string_list
      (call $Term::List::create_pair
        (global.get $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_MISSING_KEY)
        (call $Term::String::from (local.get $key)))))

  (func $Stdlib_ResolveLoaderResults::error::unexpected_key (param $key i32) (result i32)
    ;; Generate an 'unexpected key: X' label string
    (call $Term::String::collect_string_list
      (call $Term::List::create_pair
        (global.get $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_UNEXPECTED_KEY)
        (call $Term::String::from (local.get $key)))))

  (func $Stdlib_ResolveLoaderResults::error::invalid_length (param $num_expected i32) (param $num_received i32) (result i32)
    (local $segments i32)
    ;; Generate an 'expected X items, received Y' label string
    (local.tee $segments (call $Term::List::allocate (i32.const 4)))
    (call $Term::List::set_item
      (local.get $segments)
      (i32.const 0)
      (global.get $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_INVALID_LENGTH_EXPECTED))
    (call $Term::List::set_item
      (local.get $segments)
      (i32.const 1)
      (call $Term::String::from_u32 (local.get $num_expected)))
    (call $Term::List::set_item
      (local.get $segments)
      (i32.const 2)
      (global.get $Stdlib_ResolveLoaderResults::ERROR_FORMATTER_INVALID_LENGTH_RECEIVED))
    (call $Term::List::set_item
      (local.get $segments)
      (i32.const 3)
      (call $Term::String::from_u32 (local.get $num_received)))
    (call $Term::List::init (i32.const 4))
    (call $Term::String::collect_string_list)))
