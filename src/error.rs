use core::fmt;
use core::num::NonZeroU32;
use core::panic::Location;

#[non_exhaustive]
pub enum Error {
    Os(OsError),
    Skyline {
        kind: ErrorKind
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    StringTooLong
}

#[repr(transparent)]
pub struct SwitchResult(pub Option<NonZeroU32>);

pub struct OsError {
    code: u32,
    caller: &'static Location<'static>
}

impl SwitchResult {
    #[track_caller]
    pub fn ok(self) -> Result<(), OsError> {
        if let Some(code) = self.0 {
            Err(OsError {
                code: code.into(),
                caller: Location::caller()
            })
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for OsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OsError(0x{:X}) at {}:{}:{}",
            self.code,
            self.caller.file(),
            self.caller.line(),
            self.caller.column()
        )
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Self {
        Self::Os(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Os(os_err) => write!(f, "{}", os_err),
            Self::Skyline { kind } => write!(f, "{:?}", kind)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Os(os_err) => write!(f, "{}", os_err),
            Self::Skyline { kind } => write!(f, "{:?}", kind)
        }
    }
}
