fn main() -> Result<(), Box<dyn std::error::Error>> {
  lalrpop::Configuration::new()
    .use_cargo_dir_conventions()
    .emit_rerun_directives(true)
    .process_file("src/ast/parse.lalrpop")
}
