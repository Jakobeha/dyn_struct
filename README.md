
# `dyn_struct`

This crate allows you to safely initialize Dynamically Sized Types (DST) using
only safe Rust.

```rust
#[repr(C)]
#[derive(DynStruct)]
struct MyDynamicType {
    pub awesome: bool,
    pub number: u32,
    pub dynamic: [u32],
}

// the `new` function is generated by the `DynStruct` macro.
let foo: Box<MyDynamicType> = MyDynamicType::new(true, 123, &[4, 5, 6, 7]);
assert_eq!(foo.awesome, true);
assert_eq!(foo.number, 123);
assert_eq!(&foo.dynamic, &[4, 5, 6, 7]);
```


## Why Dynamic Types?

In Rust, Dynamically Sized Types (DST) are everywhere. Slices (`[T]`) and trait
objects (`dyn Trait`) are the most common ones. However, it is also possible
to define your own! For example, this can be done by letting the last field in a
struct be a dynamically sized array (note the missing `&`):

```rust
struct MyDynamicType {
    awesome: bool,
    number: u32,
    dynamic: [u32],
}
```

This tells the Rust compiler that contents of the `dynamic`-array is laid out in
memory right after the other fields. This can be very preferable in some cases,
since remove one level of indirection and increase cache-locality.

However, there's a catch! Just as with slices, the compiler does not know how
many elements are in `dynamic`. Thus, we need what is called a fat-pointer which
stores both a pointer to the actual data, but also the length of the array
itself. As of releasing this crate, the only safe way to construct a dynamic
type is if we know the size of the array at compile-time. However, for most use
cases, that is not possible. Therefore this crate uses some `unsafe` behind the
scenes to work around the limitations of the language, all wrapped up in a safe
interface.


## The Derive Macro

The `DynStruct` macro can be applied to any `#[repr(C)]` struct that contains a
dynamically sized array as its last field. Fields only have a single constraint:
they have to implement `Copy`.

### Example

```rust
#[repr(C)]
#[derive(DynStruct)]
struct MyDynamicType {
    pub awesome: bool,
    pub number: u32,
    pub dynamic: [u32],
}
```

will produce a single `impl`-block with a `new` function:

```rust
impl MyDynamicType {
    pub fn new(awesome: bool, number: u32, dynamic: &[u32]) -> Box<MyDynamicType> {
        // ... implementation details ...
    }
}
```

Due to the nature of dynamically sized types, the resulting value has to be
built on the heap. For safety reasons we currently only allow returning `Box`,
though in a future version we may also allow `Rc` and `Arc`. In the meantime it
is posible to use `Arc::from(MyDynamicType::new(...))`.
