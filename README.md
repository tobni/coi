# coi

[![Build Status](https://travis-ci.org/Nashenas88/coi.svg?branch=master)](https://travis-ci.org/Nashenas88/coi)
[![docs.rs](https://docs.rs/coi/badge.svg)](https://docs.rs/coi)
[![crates.io](https://img.shields.io/crates/v/coi.svg)](https://crates.io/crates/coi)

Dependency Injection in Rust

The goal of this crate is to provide a simple dependency injection framework
that is easy to use. Performance is not an initial concern, but might be later
on as the crate matures.

### Example 

```rust
use async_std::task;
use coi::{ContainerBuilder, Inject};
use std::sync::Arc;

pub trait Trait1: Inject {
    fn describe(&self) -> &'static str;
}

#[derive(Inject)]
#[provides(dyn Trait1 with Impl1)]
struct Impl1;

impl Trait1 for Impl1 {
    fn describe(&self) -> &'static str {
        "I'm impl1!"
    }
}

pub trait Trait2: Inject {
    fn deep_describe(&self) -> String;
}

#[derive(Inject)]
#[provides(dyn Trait2 with Impl2::new(trait1))]
struct Impl2 {
    #[inject]
    trait1: Arc<dyn Trait1>,
}

impl Impl2 {
    fn new(trait1: Arc<dyn Trait1>) -> Self {
        Self { trait1 }
    }
}

impl Trait2 for Impl2 {
    fn deep_describe(&self) -> String {
        format!("I'm impl2! and I have {}", self.trait1.describe())
    }
}

#[derive(Debug, Inject)]
#[provides(JustAStruct with JustAStruct)]
pub struct JustAStruct;

fn main() {
    task::block_on(async {
        let mut container = container!{
            trait1 => Impl1,
            trait2 => Impl2.scoped,
            struct => JustAStruct.singleton
        };
        let mut container = ContainerBuilder::new()
            .register("trait1", Impl1Provider)
            .register_as("trait2", Registration::Scoped(Impl2Provider))
            .register_as("struct", Registration::Singleton(JustAStructProvider))
            .build();
        let trait2 = container
            .resolve::<dyn Trait2>("trait2")
            .await
            .expect("Should exist");
        println!("Deep description: {}", trait2.as_ref().deep_describe());
        let a_struct = container
            .resolve::<JustAStruct>("struct")
            .await
            .expect("Should exist");
        println!("Got struct! {:?}", a_struct);
    });
}
```

#### Name

The name coi comes from an inversion of the initialism IoC (Inversion of
Control).

#### License

<sup>
Licensed under either of <a href="LICENSE.Apache-2.0">Apache License, Version
2.0</a> or <a href="LICENSE.MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

`SPDX-License-Identifier: MIT OR Apache-2.0`
