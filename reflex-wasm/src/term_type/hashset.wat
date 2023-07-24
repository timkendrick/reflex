;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Hashset
    (@struct $Hashset
      ;; TODO: implement Hashset as a standalone type rather than deferring implementation to inner Hashmap
      (@field $entries (@ref $Term)))

    (@derive $size (@get $Hashset))
    (@derive $equals (@get $Hashset))
    (@derive $hash (@get $Hashset))

    (@export $Hashset (@get $Hashset)))

  (export "isHashset" (func $Term::Hashset::is))

  (@const $Term::Hashset::EMPTY i32 (@depends-on $Term::Hashmap::EMPTY)
    (call $Term::TermType::Hashset::new (call $Term::Hashmap::empty)))

  (func $Term::Hashset::new (export "createHashset") (param $entries i32) (result i32)
    (if (result i32)
      (i32.eqz (call $Term::Hashmap::traits::length (local.get $entries)))
      (then
        (call $Term::Hashset::empty))
      (else
        (call $Term::TermType::Hashset::new (local.get $entries)))))

  (func $Term::Hashset::empty (export "createEmptyHashset") (result i32)
    (global.get $Term::Hashset::EMPTY))

  (func $Term::Hashset::traits::is_atomic (param $self i32) (result i32)
    (call $Term::Hashmap::traits::is_atomic (call $Term::Hashset::get::entries (local.get $self))))

  (func $Term::Hashset::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Hashset::traits::display (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "Set(")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u32::write_string (call $Term::Hashset::get::num_entries (local.get $self)) (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ")")
    (i32.add (local.get $offset)))

  (func $Term::Hashset::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Hashset::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Hashset::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_entries i32)
    (if (result i32)
      (i32.eq
        (local.tee $substituted_entries
          (call $Term::Hashmap::traits::substitute
            (call $Term::Hashset::get::entries (local.get $self))
            (local.get $variables)
            (local.get $scope_offset)))
        (global.get $NULL))
      (then
        (global.get $NULL))
      (else
        (call $Term::Hashset::new (local.get $substituted_entries)))))

  (func $Term::Hashset::traits::length (param $self i32) (result i32)
    (call $Term::Hashmap::traits::length (call $Term::Hashset::get::entries (local.get $self))))

  (func $Term::Hashset::traits::iterate (param $self i32) (result i32)
    (call $Term::HashmapKeysIterator::new (call $Term::Hashset::get::entries (local.get $self))))

  (func $Term::Hashset::traits::size_hint (param $self i32) (result i32)
    (call $Term::Hashmap::traits::length (call $Term::Hashset::get::entries (local.get $self))))

  (func $Term::Hashset::traits::next (param $self i32) (param $iterator_state i32) (param $state i32) (result i32 i32 i32)
    (call $Term::HashmapKeysIterator::traits::next (local.get $self) (local.get $iterator_state) (local.get $state)))

  (func $Term::Hashset::traits::has (export "hasHashsetValue") (param $self i32) (param $value i32) (result i32)
    (call $Term::Hashmap::traits::has (call $Term::Hashset::get::entries (local.get $self)) (local.get $value)))

  (func $Term::Hashset::traits::values (param $self i32) (result i32)
    (call $Term::Hashset::traits::iterate (local.get $self)))

  (func $Term::Hashset::traits::collect (param $iterator i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $Term::Hashmap::traits::collect
      ;; TODO: Avoid unnecessary heap allocations for intermediate values
      (call $Term::FlattenIterator::new
        (call $Term::ZipIterator::new
          (local.get $iterator)
          (call $Term::RepeatIterator::new (call $Term::Nil::new))))
      (local.get $state))
    (local.set $dependencies)
    (call $Term::Hashset::new)
    (local.get $dependencies))

  (func $Term::Hashset::traits::collect_strict (param $iterator i32) (param $state i32) (result i32 i32)
    (local $dependencies i32)
    (call $Term::Hashmap::traits::collect_strict
      ;; TODO: Avoid unnecessary heap allocations for intermediate values
      (call $Term::FlattenIterator::new
        (call $Term::ZipIterator::new
          (local.get $iterator)
          (call $Term::RepeatIterator::new (call $Term::Nil::new))))
      (local.get $state))
    (local.set $dependencies)
    (call $Term::Hashset::new)
    (local.get $dependencies))

  (func $Term::Hashset::get::num_entries (export "getHashsetNumEntries") (param $self i32) (result i32)
    (call $Term::Hashmap::get::num_entries (call $Term::Hashset::get::entries (local.get $self))))

  (func $Term::Hashset::push (export "pushHashsetValue") (param $self i32) (param $value i32) (result i32)
    (local $existing_entries i32)
    (local $updated_entries i32)
    (if (result i32)
      (i32.eq
        (local.tee $updated_entries
          (call $Term::Hashmap::traits::set
            (local.tee $existing_entries (call $Term::Hashset::get::entries (local.get $self)))
            (local.get $value)
            (call $Term::Nil::new)))
        (local.get $existing_entries))
      (then
        (local.get $self))
      (else
        (call $Term::Hashset::new (local.get $updated_entries))))))
