;; SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
;; SPDX-License-Identifier: Apache-2.0
;; SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
(module
  (@const-string $Stdlib_Accessor::ADD "add")
  (@const-string $Stdlib_Accessor::ENDS_WITH "endsWith")
  (@const-string $Stdlib_Accessor::ENTRIES "entries")
  (@const-string $Stdlib_Accessor::FILTER "filter")
  (@const-string $Stdlib_Accessor::FLAT_MAP "flatMap")
  (@const-string $Stdlib_Accessor::GET "get")
  (@const-string $Stdlib_Accessor::GET_TIME "getTime")
  (@const-string $Stdlib_Accessor::HAS "has")
  (@const-string $Stdlib_Accessor::JOIN "join")
  (@const-string $Stdlib_Accessor::KEYS "keys")
  (@const-string $Stdlib_Accessor::LENGTH "length")
  (@const-string $Stdlib_Accessor::MAP "map")
  (@const-string $Stdlib_Accessor::REDUCE "reduce")
  (@const-string $Stdlib_Accessor::REPLACE "replace")
  (@const-string $Stdlib_Accessor::SET "set")
  (@const-string $Stdlib_Accessor::SIZE "size")
  (@const-string $Stdlib_Accessor::SLICE "slice")
  (@const-string $Stdlib_Accessor::SPLIT "split")
  (@const-string $Stdlib_Accessor::STARTS_WITH "startsWith")
  (@const-string $Stdlib_Accessor::VALUES "values")

  (@const $Stdlib_Accessor::method::Hashmap::ENTRIES i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (call $Term::Lambda::new
      (i32.const 1)
      (call $Term::ZipIterator::new
        (call $Term::HashmapKeysIterator::new (call $Term::Variable::new (i32.const 0)))
        (call $Term::HashmapValuesIterator::new (call $Term::Variable::new (i32.const 0))))))

  (@const $Stdlib_Accessor::method::<iterate>::FILTER i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (@depends-on $Stdlib_Accessor::method::<iterate>::FILTER_PREDICATE)
    (call $Term::Lambda::new
      (i32.const 2)
      (call $Term::FilterIterator::new
        (call $Term::Variable::new (i32.const 1))
        (call $Term::Partial::new
          (global.get $Stdlib_Accessor::method::<iterate>::FILTER_PREDICATE)
          (call $Term::List::of (call $Term::Variable::new (i32.const 0)))))))

  (@const $Stdlib_Accessor::method::<iterate>::FILTER_PREDICATE i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (call $Term::Lambda::new
      (i32.const 2)
      (call $Term::Application::new
        (call $Term::Builtin::new (global.get $Stdlib_IsTruthy))
        (call $Term::List::of
          (call $Term::Application::new
            (call $Term::Variable::new (i32.const 1))
            (call $Term::List::of (call $Term::Variable::new (i32.const 0))))))))

  (@const $Stdlib_Accessor::method::<iterate>::FLAT_MAP i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (call $Term::Lambda::new
      (i32.const 2)
      (call $Term::FlattenIterator::new
        (call $Term::MapIterator::new
          (call $Term::Variable::new (i32.const 1))
          (call $Term::Variable::new (i32.const 0))))))

  (@const $Stdlib_Accessor::method::<iterate>::KEYS i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::IntegersIterator::INSTANCE)
    (@depends-on $Stdlib_Accessor::SELECT_FIRST)
    (call $Term::Lambda::new
      (i32.const 1)
      (call $Term::MapIterator::new
        (call $Term::ZipIterator::new
          (call $Term::IntegersIterator::new)
          (call $Term::Variable::new (i32.const 0)))
        (global.get $Stdlib_Accessor::SELECT_FIRST))))

  (@const $Stdlib_Accessor::SELECT_FIRST i32
    (@depends-on $Term::Int::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_0)
    (call $Term::Lambda::new
      (i32.const 1)
      (call $Term::Application::new
        (call $Term::Builtin::new (global.get $Stdlib_Get))
        (call $Term::List::create_pair
          (call $Term::Variable::new (i32.const 0))
          (call $Term::Int::new (i64.const 0))))))

  (@const $Stdlib_Accessor::method::<iterate>::JOIN i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (call $Term::Lambda::new
      (i32.const 2)
      (call $Term::Application::new
        (call $Term::Builtin::new (global.get $Stdlib_Apply))
        (call $Term::List::create_pair
          (call $Term::Builtin::new (global.get $Stdlib_CollectString))
          (call $Term::IntersperseIterator::new
            (call $Term::MapIterator::new
              (call $Term::Variable::new (i32.const 1))
              (call $Term::Builtin::new (global.get $Stdlib_ToString)))
            (call $Term::Variable::new (i32.const 0)))))))

  (@const $Stdlib_Accessor::method::<iterate>::MAP i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (call $Term::Lambda::new
      (i32.const 2)
      (call $Term::MapIterator::new
        (call $Term::Variable::new (i32.const 1))
        (call $Term::Variable::new (i32.const 0)))))

  (@const $Stdlib_Accessor::method::<iterate>::REDUCE i32
    (@depends-on $Term::Variable::INSTANCE_0)
    (@depends-on $Term::Variable::INSTANCE_1)
    (@depends-on $Term::Variable::INSTANCE_2)
    (call $Term::Lambda::new
      (i32.const 3)
      (call $Term::Application::new
        (call $Term::Builtin::new (global.get $Stdlib_Fold))
        (call $Term::List::create_triple
          (call $Term::Variable::new (i32.const 2))
          (call $Term::Variable::new (i32.const 0))
          (call $Term::Variable::new (i32.const 1))))))

  (@builtin $Stdlib_Accessor "Accessor"
    (@args (@strict $self) (@strict $key))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::Record::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $value (call $Term::Record::traits::get (local.get $self) (local.get $key)))
            (global.get $NULL))
          (then
            (local.get $value)
            (global.get $NULL))
          (else
            ;; Default to returning an error for unrecognized field names
            (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::Record))
      (i32.eq (global.get $TermType::Symbol))
      (func $Stdlib_Accessor::impl::Record::Symbol (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $value (call $Term::Record::traits::get (local.get $self) (local.get $key)))
            (global.get $NULL))
          (then
            (local.get $value)
            (global.get $NULL))
          (else
            ;; Default to returning an error for unrecognized field names
            (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Accessor::impl::List::Int (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (local $index_value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $value
              (if (result i32)
                (i32.lt_u
                  (local.tee $index_value (i32.wrap_i64 (call $Term::Int::get_value (local.get $index))))
                  (call $Term::List::get_length (local.get $self)))
                (then
                  (call $Term::List::get_item (local.get $self) (local.get $index_value)))
                (else
                  (global.get $NULL))))
            (global.get $NULL))
          (then
            (local.get $value)
            (global.get $NULL))
          (else
            (call $Term::Nil::new)
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Accessor::impl::List::Float (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $index_value i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $index_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $index))))
            (i32.const -1))
          (then
            (if (result i32 i32)
              (i32.ne
                (local.tee $value
                  (if (result i32)
                    (i32.lt_u (local.get $index_value) (call $Term::List::get_length (local.get $self)))
                    (then
                      (call $Term::List::get_item (local.get $self) (local.get $index_value)))
                    (else
                      (global.get $NULL))))
                (global.get $NULL))
              (then
                (local.get $value)
                (global.get $NULL))
              (else
                (call $Term::Nil::new)
                (global.get $NULL))))
          (else
            (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $index) (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Accessor::impl::String::Int (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $index_value i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $value
              (if (result i32)
                (i32.lt_u
                  (local.tee $index_value (i32.wrap_i64 (call $Term::Int::get_value (local.get $index))))
                  (call $Term::String::get_length (local.get $self)))
                (then
                  (call $Term::String::get_char (local.get $self) (local.get $index_value)))
                (else
                  (global.get $NULL))))
            (global.get $NULL))
          (then
            (local.get $value)
            (global.get $NULL))
          (else
            (call $Term::Nil::new)
            (global.get $NULL)))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Accessor::impl::String::Float (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $index_value i32)
        (local $value i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $index_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $index))))
            (i32.const -1))
          (then
            (if (result i32 i32)
              (i32.ne
                (local.tee $value
                  (if (result i32)
                    (i32.lt_u (local.get $index_value) (call $Term::String::get_length (local.get $self)))
                    (then
                      (call $Term::String::get_char (local.get $self) (local.get $index_value)))
                    (else
                      (global.get $NULL))))
                (global.get $NULL))
              (then
                (local.get $value)
                (global.get $NULL))
              (else
                (call $Term::Nil::new)
                (global.get $NULL))))
          (else
            (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $index) (local.get $state))))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Int))
      (func $Stdlib_Accessor::impl::<iterate>::Int (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $value i32)
        (local $dependencies i32)
        (call $Term::traits::next
          (call $Term::SkipIterator::new (local.get $self) (i32.wrap_i64 (call $Term::Int::get_value (local.get $index))))
          (global.get $NULL)
          (local.get $state))
        (local.set $dependencies)
        (drop)
        (if (result i32 i32)
          (i32.ne (local.tee $value) (global.get $NULL))
          (then
            (local.get $value)
            (global.get $NULL))
          (else
            (call $Term::Nil::new)
            (global.get $NULL)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::Float))
      (func $Stdlib_Accessor::impl::<iterate>::Float (param $self i32) (param $index i32) (param $state i32) (result i32 i32)
        (local $index_value i32)
        (local $value i32)
        (local $dependencies i32)
        (if (result i32 i32)
          (i32.ne
            (local.tee $index_value (i32.wrap_i64 (call $Term::Float::get_non_negative_integer_value (local.get $index))))
            (i32.const -1))
          (then
            (call $Term::traits::next
              (call $Term::SkipIterator::new (local.get $self) (local.get $index_value))
              (global.get $NULL)
              (local.get $state))
            (local.set $dependencies)
            (drop)
            (if (result i32 i32)
              (i32.ne (local.tee $value) (global.get $NULL))
              (then
                (local.get $value)
                (global.get $NULL))
              (else
                (call $Term::Nil::new)
                (global.get $NULL))))
          (else
            (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $index) (local.get $state))))))

    (@impl
      (i32.eq (global.get $TermType::String))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::String::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::ENDS_WITH))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_EndsWith))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::LENGTH))
              (return
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_Length))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::REPLACE))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Replace))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::SPLIT))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Split))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::STARTS_WITH))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_StartsWith))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to returning an error for unrecognized member names
          (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state)))))

    (@impl
      (i32.eq (global.get $TermType::Hashmap))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::Hashmap::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::ENTRIES))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::Hashmap::ENTRIES)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::GET))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Get))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::HAS))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Has))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::KEYS))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Keys))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::SET))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Set))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::SIZE))
              (return
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_Length))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::VALUES))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Values))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to returning an error for unrecognized member names
          (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state)))))

    (@impl
      (i32.eq (global.get $TermType::Hashset))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::Hashset::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::ADD))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Push))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::ENTRIES))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Values))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::HAS))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Has))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::KEYS))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Keys))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::SIZE))
              (return
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_Length))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::VALUES))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Values))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to returning an error for unrecognized member names
          (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state)))))

    (@impl
      (i32.eq (global.get $TermType::List))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::List::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::KEYS))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Keys))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to the generic iterator implementation
          (call $Stdlib_Accessor::impl::<iterate>::String (local.get $self) (local.get $key) (local.get $state)))))

    (@impl
      (i32.eq (global.get $TermType::Timestamp))
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::Timestamp::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::GET_TIME))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_ParseInt))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to returning an error for unrecognized member names
          (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state)))))

    (@impl
      (call $TermType::implements::iterate)
      (i32.eq (global.get $TermType::String))
      (func $Stdlib_Accessor::impl::<iterate>::String (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (@switch
          ;; Determine the return value based on the provided member name
          (@list
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::ENTRIES))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Values))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::FILTER))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::FILTER)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::FLAT_MAP))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::FLAT_MAP)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::KEYS))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::KEYS)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::JOIN))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::JOIN)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::LENGTH))
              (return
                (call $Term::Application::new
                  (call $Term::Builtin::new (global.get $Stdlib_Length))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::MAP))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::MAP)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::REDUCE))
              (return
                (call $Term::Partial::new
                  (global.get $Stdlib_Accessor::method::<iterate>::REDUCE)
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::SLICE))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Slice))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL)))
            (@list
              (call $Term::traits::equals (local.get $key) (global.get $Stdlib_Accessor::VALUES))
              (return
                (call $Term::Partial::new
                  (call $Term::Builtin::new (global.get $Stdlib_Values))
                  (call $Term::List::of (local.get $self)))
                (global.get $NULL))))
          ;; Default to returning an error for unrecognized member names
          (call $Stdlib_Accessor::impl::default (local.get $self) (local.get $key) (local.get $state)))))

    (@default
      (func $Stdlib_Accessor::impl::default (param $self i32) (param $key i32) (param $state i32) (result i32 i32)
        (call $Term::Signal::of
          (call $Term::Condition::invalid_builtin_function_args
            (global.get $Stdlib_Accessor)
            (call $Term::List::create_pair (local.get $self) (local.get $key))))
        (global.get $NULL)))))
