;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@let $Dependency
    (@union $Dependency
      (@struct $CacheDependency
        (@field $key i64))

      (@struct $StateDependency
        (@field $condition (@ref $Term @optional))))

    (@derive $size (@get $Dependency))
    (@derive $equals (@get $Dependency))
    (@derive $hash (@get $Dependency))
    (@map $typename
      (@union_variants (@get $Dependency))
      (@block
        (@derive $size (@union_variant (@get $Dependency) (@get $_)))
        (@derive $equals (@union_variant (@get $Dependency) (@get $_)))
        (@derive $hash (@union_variant (@get $Dependency) (@get $_)))))

    (@export $Dependency (@get $Dependency))

    ;; Declare global term type constants
    (@map $typename
      (@union_variants (@get $Dependency))
      (@block
        (global (@concat "$Dependency::" (@get $typename)) (export (@concat "\"" "DependencyType_" (@get $typename) "\"")) i32 (i32.const (@get $_)))

        (func (@concat "$Dependency::" (@get $typename) "::sizeof") (result i32)
          (i32.add
            ;; Add 4 bytes for the discriminant
            (i32.const 4)
            ;; Add the size of the underlying type variant
            (call (@concat "$" (@get $typename) "::sizeof"))))))

    ;; Generate display formatters for all type variants
    (func $DependencyType::display (param $variant i32) (param $offset i32) (result i32)
      (@branch
        (local.get $variant)
        (@list
          (@map $typename
            (@union_variants (@get $Dependency))
            (return (call (@concat "$DependencyType::" (@get $typename) "::display") (local.get $offset)))))
        (local.get $offset)))

    (@map $typename
      (@union_variants (@get $Dependency))
      (@block
        (func (@concat "$DependencyType::" (@get $typename) "::display") (param $offset i32) (result i32)
          (@store-bytes $offset (@to-string (@get $typename)))
          (i32.add (local.get $offset))))))

  (export "isDependency" (func $Term::Dependency::is))
  (export "getDependencyType" (func $Term::Dependency::get::type))

  (func $Term::Dependency::cache (export "createCacheDependency") (param $key i64) (result i32)
    (call $Term::TermType::Dependency::CacheDependency::new (local.get $key)))

  (func $Term::Dependency::state (export "createStateDependency") (param $condition i32) (result i32)
    (call $Term::TermType::Dependency::StateDependency::new (local.get $condition)))

  (func $Term::Dependency::traits::is_atomic (param $self i32) (result i32)
    (global.get $FALSE))

  (func $Term::Dependency::traits::is_truthy (param $self i32) (result i32)
    (global.get $TRUE))

  (func $Term::Dependency::traits::display (param $self i32) (param $offset i32) (result i32)
    (local $type i32)
    (local.set $type (call $Term::Dependency::get::type (local.get $self)))
    (block $BLOCK
      (@switch
        (@list
          (@list
            (i32.eq (local.get $type) (global.get $Dependency::CacheDependency))
            (block
              (local.set $offset
                (call $Term::Dependency::CacheDependency::traits::debug
                  (local.get $self)
                  (local.get $offset)))
              (br $BLOCK)))
          (@list
            (i32.eq (local.get $type) (global.get $Dependency::StateDependency))
            (block
              (local.set $offset
                (call $Term::Dependency::StateDependency::traits::debug
                  (local.get $self)
                  (local.get $offset)))
              (br $BLOCK))))))
    (local.get $offset))

  (func $Term::Dependency::traits::debug (param $self i32) (param $offset i32) (result i32)
    (call $Term::Dependency::traits::display (local.get $self) (local.get $offset)))

  (func $Term::Dependency::CacheDependency::traits::debug (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "<#")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Utils::u64::write_string
      (call $Term::Dependency::CacheDependency::get::key (local.get $self))
      (local.get $offset))
    (local.set $offset (i32.add (local.get $offset)))
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::Dependency::StateDependency::traits::debug (param $self i32) (param $offset i32) (result i32)
    (@store-bytes $offset "<")
    (local.set $offset (i32.add (local.get $offset)))
    (call $Term::traits::display
      (call $Term::Dependency::StateDependency::get::condition (local.get $self))
      (local.get $offset))
    (local.set $offset)
    (@store-bytes $offset ">")
    (i32.add (local.get $offset)))

  (func $Term::Dependency::CacheDependency::get::key (export "getCacheDependencyKey") (param $self i32) (result i64)
    (call $Term::Dependency::get::value (local.get $self))
    (call $CacheDependency::get::key))

  (func $Term::Dependency::StateDependency::get::condition (export "getStateDependencyCondition") (param $self i32) (result i32)
    (call $Term::Dependency::get::value (local.get $self))
    (call $StateDependency::get::condition))

  (func $Term::Dependency::traits::substitute (param $self i32) (param $variables i32) (param $scope_offset i32) (result i32)
    (local $substituted_keys i32)
    (local.set $substituted_keys
      (call $Term::traits::substitute
        (call $Term::Constructor::get::keys (local.get $self))
        (local.get $variables)
        (local.get $scope_offset)))
    (if (result i32)
      (i32.eq (global.get $NULL) (local.get $substituted_keys))
      (then
        (global.get $NULL))
      (else
        (call $Term::Constructor::new
          (local.get $substituted_keys))))))
