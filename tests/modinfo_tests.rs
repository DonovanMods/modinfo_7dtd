use std::borrow::Cow;

#[cfg(test)]
mod fixtures;

#[test]
fn test_modinfo_parse() {
    let modinfo = modinfo::parse(fixtures::setup()).unwrap();

    assert_eq!(
        modinfo.get_value_for("name"),
        Some(&Cow::from("SomeInternalName"))
    );
    assert_eq!(
        modinfo.get_value_for("display_name"),
        Some(&Cow::from("Official Mod Name"))
    );
    assert_eq!(modinfo.get_version().to_string(), "1.2.3".to_owned());
    assert_eq!(modinfo.get_value_for("compat"), Some(&Cow::from("A99")));
    assert_eq!(
        modinfo.get_value_for("author"),
        Some(&Cow::from("Author Name"))
    );
    assert_eq!(
        modinfo.get_value_for("description"),
        Some(&Cow::from("Mod to show format of ModInfo v2"))
    );
    assert_eq!(
        modinfo.get_value_for("website"),
        Some(&Cow::from("https://example.org"))
    );
    assert_eq!(modinfo.get_value_for("foo"), None);

    fixtures::cleanup();
}
