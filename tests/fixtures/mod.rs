use std::{fs, path};

fn create_modinfo_file_v2() -> path::PathBuf {
    std::fs::write(
        "tests/fixtures/modinfo_v2.xml",
        r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <xml>
                <Name value="SomeInternalName" />
                <DisplayName value="Official Mod Name" />
                <Version value="1.2.3" compat="A99" />
                <Description value="Mod to show format of ModInfo v2" />
                <Author value="Author Name" />
                <Website value="https://example.org" />
            </xml>
        "#,
    )
    .unwrap();

    path::PathBuf::from("tests/fixtures/modinfo_v2.xml")
}

pub fn setup() -> path::PathBuf {
    create_modinfo_file_v2()
}

pub fn cleanup() {
    if path::Path::new("tests/fixtures/modinfo_v2.xml").exists() {
        fs::remove_file("tests/fixtures/modinfo_v2.xml").unwrap();
    }
}
