use modinfo::*;

#[test]
fn get_value_for_test() {
    let modinfo = Modinfo::parse(&fixtures::xml_string_v2()).unwrap();

    assert_eq!(
        modinfo.get_value_for("name"),
        Some(&String::from("SomeInternalName"))
    );
    assert_eq!(
        modinfo.get_value_for("display_name"),
        Some(&String::from("Official Mod Name"))
    );
    assert_eq!(modinfo.get_value_for("author"), Some(&String::from("Name")));
    assert_eq!(modinfo.get_value_for("compat"), Some(&String::from("A99")));
    assert_eq!(
        modinfo.get_value_for("description"),
        Some(&String::from("Mod to show format of ModInfo v2"))
    );
    assert_eq!(modinfo.get_value_for("website"), Some(&String::from("HP")));
    assert_eq!(modinfo.get_value_for("foo"), None);
}
