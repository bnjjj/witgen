#[witgen::witgen]
pub type StringAlias = String;

mod private {
    #[witgen::witgen]
    type PrivateType = Vec<f32>;
    mod second_level {
        #[witgen::witgen]
        type SecondLevel = bool;
    }
}
