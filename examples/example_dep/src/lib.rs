#![allow(dead_code)]

/// Example of an external dependency
#[witgen::witgen]
pub type ExternalDep = String;

pub struct SampleResource {}

/// Example Interface
#[witgen::witgen]
impl SampleResource {
  /// Can handle static methods
  pub fn faa() {}
 
  ///Can add special comments
  #[payable]
  pub fn foo(&self) -> String {
    "foo".to_string()
  }

  /// Can indicate if mutable
  pub fn f(&mut self) -> () { () }

}

#[witgen::witgen]
fn faa() {}

#[witgen::witgen]
pub enum OtherColors {
  Orange,
  Purple,
  Black,
}
