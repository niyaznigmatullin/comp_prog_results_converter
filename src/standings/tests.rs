pub(crate) mod tests {
    use std::fs::read_to_string;
    use std::path::PathBuf;

    pub(crate) fn read_resource(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/resources")
            .join(name);
        read_to_string(path).unwrap()
    }
}
