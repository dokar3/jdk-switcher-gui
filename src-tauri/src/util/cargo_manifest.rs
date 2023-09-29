pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: String,
}

const CARGO_TOML_STR: &str = include_str!("../../Cargo.toml");

pub fn read_cargo_package() -> CargoPackage {
    let table = CARGO_TOML_STR.parse::<toml::Table>().unwrap();
    let package = table.get("package").unwrap();
    let str_value_of = |key: &str| package.get(key).unwrap().as_str().unwrap().to_string();
    CargoPackage {
        name: str_value_of("name"),
        version: str_value_of("version"),
        description: str_value_of("description"),
        repository: str_value_of("repository"),
    }
}
