#![allow(dead_code)]

/// Example of an external dependency
#[witgen::witgen]
pub type ExternalDep = String;

pub struct SampleResource {}

/// Example Interface
#[witgen::witgen]
impl SampleResource {
    #[payable]
    pub fn foo(&mut self) -> String {
        "foo".to_string()
    }
}
