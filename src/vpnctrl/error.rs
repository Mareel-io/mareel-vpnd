use custom_error::custom_error;

custom_error! {pub VpnctrlError
    BadParameterError{msg: String} = "Bad parameter: {msg}",
    DuplicatedEntryError{msg: String} = "Duplicated entry: {msg}",
    EntryNotFoundError{msg: String} = "Entry not found: {msg}",
    InternalError{msg: String} = "Internal error: {msg}",
}
