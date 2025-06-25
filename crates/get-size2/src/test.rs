#![expect(dead_code, clippy::unwrap_used, reason = "This is a test module")]

use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::OnceLock;

use get_size2::*;

#[derive(GetSize)]
pub struct TestStruct {
    value1: String,
    value2: u64,
}

#[test]
fn derive_struct() {
    let test = TestStruct {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
pub struct TestStructGenerics<A, B> {
    value1: A,
    value2: B,
}

#[test]
fn derive_struct_with_generics() {
    let test: TestStructGenerics<String, u64> = TestStructGenerics {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
#[get_size(ignore(B, C))]
struct TestStructGenericsIgnore<A, B, C> {
    value1: A,
    #[get_size(ignore)]
    value2: B,
    #[get_size(ignore)]
    value3: C,
}

struct TestStructNoGetSize {
    value: String,
}

#[test]
fn derive_struct_with_generics_and_ignore() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructGenericsIgnore<String, u64, TestStructNoGetSize> =
        TestStructGenericsIgnore {
            value1: "Hello".into(),
            value2: 123,
            value3: no_impl,
        };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
#[get_size(ignore(B, C))]
struct TestStructHelpers<A, B, C> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = get_size_helper)]
    value3: C,
}

const fn get_size_helper<C>(_value: &C) -> usize {
    50
}

#[test]
fn derive_struct_with_generics_and_helpers() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructHelpers<String, u64, TestStructNoGetSize> = TestStructHelpers {
        value1: "Hello".into(),
        value2: 123,
        value3: no_impl,
    };

    assert_eq!(test.get_heap_size(), 5 + 100 + 50);
}

#[derive(GetSize)]
pub struct TestStructGenericsLifetimes<'a, A, B> {
    value1: A,
    value2: &'a B,
}

#[test]
fn derive_struct_with_generics_and_lifetimes() {
    let value = 123u64;

    let test: TestStructGenericsLifetimes<'_, String, u64> = TestStructGenericsLifetimes {
        value1: "Hello".into(),
        value2: &value,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
pub enum TestEnum {
    Variant1(u8, u16, u32),
    Variant2(String),
    Variant3(i64, Vec<u16>),
    Variant4(String, i32, Vec<u32>, bool, &'static str),
    Variant5(f64, TestStruct),
    Variant6,
    Variant7 { x: String, y: String },
}

#[test]
fn derive_enum() {
    let test = TestEnum::Variant1(1, 2, 3);
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test = TestEnum::Variant3(-12, vec![1, 2, 3]);
    assert_eq!(test.get_heap_size(), 6);

    let s: String = "Test".into();
    assert_eq!(s.get_heap_size(), 4);
    let v = vec![1, 2, 3, 4];
    assert_eq!(v.get_heap_size(), 16);
    let test = TestEnum::Variant4(s, -123, v, false, "Hello world!");
    assert_eq!(test.get_heap_size(), 4 + 16);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test = TestEnum::Variant5(12.34, test_struct);
    assert_eq!(test.get_heap_size(), 11);

    let test = TestEnum::Variant6;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant7 {
        x: "Hello".into(),
        y: "world".into(),
    };
    assert_eq!(test.get_heap_size(), 5 + 5);
}

#[derive(GetSize)]
pub enum TestEnumGenerics<'a, A, B, C> {
    Variant1(A),
    Variant2(B),
    Variant3(&'a C),
}

#[test]
fn derive_enum_generics() {
    let test: TestEnumGenerics<'_, u64, String, TestStruct> = TestEnumGenerics::Variant1(123);
    assert_eq!(test.get_heap_size(), 0);

    let test: TestEnumGenerics<'_, u64, String, TestStruct> =
        TestEnumGenerics::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test: TestEnumGenerics<'_, u64, String, TestStruct> =
        TestEnumGenerics::Variant3(&test_struct);
    assert_eq!(test.get_heap_size(), 0); // It is a pointer.
}

const MINIMAL_NODE_SIZE: usize = 3;

#[derive(Clone, GetSize)]
enum Node<T>
where
    T: Default,
{
    Block(T),
    Blocks(Box<[T; MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE]>),
    Nodes(Box<[Node<T>; 8]>),
}

#[test]
fn derive_enum_generics_issue1() {
    let test: Node<String> = Node::Block("test".into());
    assert_eq!(test.get_heap_size(), 4);

    let test: Node<u64> = Node::Blocks(Box::new([123; 27]));
    assert_eq!(test.get_heap_size(), 8 * 27);

    let t1: Node<u64> = Node::Block(123);
    let t2 = t1.clone();
    let t3 = t1.clone();
    let t4 = t1.clone();
    let t5 = t1.clone();
    let t6 = t1.clone();
    let t7 = t1.clone();
    let t8 = t1.clone();
    let test: Node<u64> = Node::Nodes(Box::new([t1, t2, t3, t4, t5, t6, t7, t8]));
    assert_eq!(test.get_heap_size(), 8 * std::mem::size_of::<Node<u64>>());
}

#[derive(GetSize)]
pub enum TestEnum2 {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[test]
fn derive_enum_c_style() {
    let test = TestEnum2::Zero;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum2::One;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum2::Two;
    assert_eq!(test.get_heap_size(), 0);
}

#[derive(GetSize)]
pub struct TestNewType(u64);

#[test]
fn derive_newtype() {
    let test = TestNewType(0);
    assert_eq!(u64::get_stack_size(), test.get_size());
}

#[test]
fn boxed_slice() {
    use std::mem::size_of;
    let boxed = vec![1u8; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<u8>() * boxed.len());

    let boxed = vec![1u32; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<u32>() * boxed.len());

    let boxed = vec![&1u8; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<&u8>() * boxed.len());
}

#[test]
fn boxed_str() {
    let boxed: Box<str> = "a".repeat(1).into();
    assert_eq!(boxed.get_heap_size(), size_of::<u8>() * boxed.len());

    let rc: Rc<str> = "a".repeat(1).into();
    assert_eq!(rc.get_heap_size(), size_of::<u8>() * boxed.len());

    let arc: Arc<str> = "a".repeat(1).into();
    assert_eq!(arc.get_heap_size(), size_of::<u8>() * boxed.len());
}

#[test]
fn chrono() {
    use chrono::TimeZone;

    let timedelta = chrono::TimeDelta::seconds(5);
    assert_eq!(timedelta.get_heap_size(), 0);

    let datetime = chrono::Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`
    assert_eq!(datetime.naive_utc().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().date().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().time().get_heap_size(), 0);
    assert_eq!(datetime.timezone().get_heap_size(), 0);
    assert_eq!(datetime.fixed_offset().timezone().get_heap_size(), 0);
    assert_eq!(datetime.get_heap_size(), 0);
}

#[test]
fn chrono_tz() {
    use chrono::TimeZone;

    let datetime = chrono_tz::UTC
        .with_ymd_and_hms(2014, 7, 8, 9, 10, 11)
        .unwrap(); // `2014-07-08T09:10:11Z`
    assert_eq!(datetime.offset().get_heap_size(), 0);
}

#[test]
fn url() {
    const URL_STR: &str = "https://example.com/path?a=b&c=d";

    let url = url::Url::parse(URL_STR).unwrap();
    assert_eq!(url.get_heap_size(), URL_STR.len());
}

#[test]
fn bytes() {
    const BYTES_STR: &str = "Hello world";

    let bytes = bytes::Bytes::from(BYTES_STR);
    assert_eq!(bytes.get_heap_size(), BYTES_STR.len());

    let mut bytes_mut = bytes::BytesMut::from(BYTES_STR);
    assert_eq!(bytes_mut.get_heap_size(), BYTES_STR.len());
    bytes_mut.truncate(0);
    assert_eq!(bytes_mut.get_heap_size(), 0);
}

fn once_lock_get_size() {
    // empty OnceLock
    let lock: OnceLock<String> = OnceLock::new();
    assert_eq!(lock.get_heap_size(), 0);

    // filled OnceLock
    let lock_filled: OnceLock<String> = {
        let l = OnceLock::new();
        l.set(String::from("HalloTest")).unwrap();
        l
    };
    // The heap size of a OnceLock filled with a String is the size of the String's heap allocation.
    assert_eq!(
        lock_filled.get_heap_size(),
        lock_filled.get().unwrap().capacity()
    );
}

fn compact_str() {
    const STR: &str = "Hello world";
    const LONG_STR: &str = "A much looooonger string.";

    let value = compact_str::CompactString::from(STR);
    assert_eq!(value.get_heap_size(), 0);

    let value = compact_str::CompactString::from(LONG_STR);
    assert_eq!(value.get_heap_size(), LONG_STR.len());
}

#[test]
fn hashbrown() {
    use std::hash::{BuildHasher, RandomState};

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = hashbrown::HashTable::new();
    assert_eq!(map.get_heap_size(), 0);
    map.insert_unique(
        hasher.hash_one(&VALUE_STR),
        String::from(VALUE_STR),
        |value| hasher.hash_one(&value),
    );
    assert!(map.get_heap_size() >= size_of::<String>() + VALUE_STR.len());

    let mut map = hashbrown::HashMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = hashbrown::HashSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn smallvec() {
    const ITEM_STR: &str = "Hello world";
    let mut vec = smallvec::SmallVec::<[String; 2]>::from([String::new(), String::from(ITEM_STR)]);

    assert_eq!(vec.get_heap_size(), ITEM_STR.len());
    vec.push(String::new());

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len() + std::mem::size_of::<String>() * 3
    );
}
