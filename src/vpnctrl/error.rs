use core::fmt;

pub trait VpnctrlError {}

#[derive(Debug, Clone)]
pub struct BadParameterError {
    msg: String,
}

impl fmt::Display for BadParameterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BadParameterError: {}", self.msg)
    }
}

impl VpnctrlError for BadParameterError {}

impl BadParameterError {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DuplicatedEntryError {
    msg: String,
}

impl fmt::Display for DuplicatedEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BadParameterError: {}", self.msg)
    }
}

impl VpnctrlError for DuplicatedEntryError {}

impl DuplicatedEntryError {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}
