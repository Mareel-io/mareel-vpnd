use custom_error::custom_error;

custom_error! {pub VpnctrlError
    BadParameter{msg: String} = "Bad parameter: {msg}",
    DuplicatedEntry{msg: String} = "Duplicated entry: {msg}",
    EntryNotFound{msg: String} = "Entry not found: {msg}",
    Internal{msg: String} = "Internal error: {msg}",
}
