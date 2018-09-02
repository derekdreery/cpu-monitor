#[cfg(feature = "ci")]
extern crate skeptic;

fn main() {
    #[cfg(feature = "ci")]
    skeptic::generate_doc_tests(&["README.md"]);
}
