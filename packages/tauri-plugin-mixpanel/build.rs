const COMMANDS: &[&str] = &[
    "register",
    "register_once",
    "unregister",
    "identify",
    "alias",
    "track",
    "get_distinct_id",
    "get_property",
    "reset",
    "time_event",
    "set_group",
    "add_group",
    "remove_group",
    "people_set",
    "people_set_once",
    "people_unset",
    "people_increment",
    "people_append",
    "people_remove",
    "people_union",
    "people_delete_user",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
