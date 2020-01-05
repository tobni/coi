use coi::{container, Inject};
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

#[test]
fn main() {
    async_std::task::block_on(async {
        let mut container = container! {
            trait1 => Impl1Provider,
            trait2 => Impl2Provider,
            struct => JustAStructProvider,
        };
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