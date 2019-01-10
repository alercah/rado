fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=src/ast/parse.lalrpop");
  lalrpop::Configuration::new()
    .use_cargo_dir_conventions()
    .process_file("src/ast/parse.lalrpop")
}
