use testify::chromosome::TestCase;
use generation::source::SourceFile;

#[test]
fn test_structs() {
    let mut source_file = SourceFile::<TestCase>::new("tests/example.rs");
    source_file.instrument();
    println!("{:?}", source_file.structs());
}