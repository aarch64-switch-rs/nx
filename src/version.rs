use crate::sync;
use core::cmp;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8
}

impl Version {
    pub const fn empty() -> Self {
        Self { major: 0, minor: 0, micro: 0 }
    }

    pub const fn new(major: u8, minor: u8, micro: u8) -> Self {
        Self { major: major, minor: minor, micro: micro }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self.major < other.major {
            cmp::Ordering::Less
        }
        else if self.major == other.major {
            if self.minor < other.minor {
                cmp::Ordering::Less
            }
            else if self.minor == other.minor {
                if self.micro < other.micro {
                    cmp::Ordering::Less
                }
                else {
                    cmp::Ordering::Equal
                }
            }
            else {
                cmp::Ordering::Greater
            }
        }
        else {
            cmp::Ordering::Greater
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

static mut G_VERSION: sync::Locked<Version> = sync::Locked::new(false, Version::empty());

pub(crate) fn set_version(version: Version) {
    unsafe {
        G_VERSION.set(version);
    }
}

pub fn get_version() -> Version {
    unsafe {
        *G_VERSION.get()
    }
}