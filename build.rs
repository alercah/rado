fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("output dir: {}", std::env::var("OUT_DIR").unwrap());
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process()
}
