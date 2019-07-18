use gltf_codegen::Wrapper;
use std::marker;

#[derive(Debug)]
struct Buffer;

#[derive(Debug)]
struct Index<T>(u32, marker::PhantomData<T>);

impl<T> Index<T> {
    pub fn new(index: u32) -> Self {
        Index(index, marker::PhantomData)
    }
}

/// Documentation here.
#[allow(dead_code)]
#[derive(Debug, Wrapper)]
struct InputStruct {
    pub buffer: Index<Buffer>,
    pub foo: u32,
}

#[test]
fn works() {
    let generated = GeneratedStruct { buffer: Index::new(123), foo: 456 };
    println!("{:#?}", generated);
}
