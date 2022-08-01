use dyn_struct2::dyn_arg;
use dyn_struct_derive2::DynStruct;

#[test]
fn custom() {
    #[repr(C)]
    #[derive(Debug, DynStruct)]
    struct Foo {
        pub inner: u32,
        pub values: [u32],
    }

    let foo = Foo::new(14, dyn_arg!([1, 2, 3, 4]));
    assert_eq!(foo.inner, 14);
    assert_eq!(&foo.values, [1, 2, 3, 4]);
}

#[test]
fn generic() {
    #[repr(C)]
    #[derive(Debug, DynStruct)]
    struct Foo<'a, T: Copy, U: Copy> {
        pub inner: T,
        pub text: &'a str,
        pub values: [U],
    }

    let foo = Foo::new(true, "hello", dyn_arg!([1, 2, 3, 4]));
    assert_eq!(foo.inner, true);
    assert_eq!(foo.text, "hello");
    assert_eq!(&foo.values, [1, 2, 3, 4]);
}

#[test]
fn readme() {
    #[repr(C)]
    #[derive(DynStruct)]
    struct MyDynamicType {
        pub awesome: bool,
        pub number: u32,
        pub dynamic: [u32],
    }

    let foo: Box<MyDynamicType> = MyDynamicType::new(true, 123, dyn_arg!([4, 5, 6, 7, 8]));
    assert_eq!(foo.awesome, true);
    assert_eq!(foo.number, 123);
    assert_eq!(&foo.dynamic, &[4, 5, 6, 7, 8]);
}

