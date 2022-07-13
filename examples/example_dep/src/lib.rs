/// Example of an external dependency
#[witgen::witgen]
pub type ExternalDep = String;


/// Example Interface
#[witgen::witgen]
pub trait SampleResource {
  fn foo() -> String {
    "foo".to_string()
  }
}