use core::fmt;
use core::num::NonZeroU32;
use core::panic::Location;
use core::str;

use crate::c_str;
use crate::nn;

#[non_exhaustive]
pub enum Error {
    Os(OsError),
    Skyline { kind: ErrorKind },
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    StringTooLong,
}

#[repr(transparent)]
pub struct SwitchResult(pub Option<NonZeroU32>);

pub struct OsError {
    code: u32,
    caller: &'static Location<'static>,
}

impl SwitchResult {
    #[track_caller]
    pub fn ok(self) -> Result<(), OsError> {
        if let Some(code) = self.0 {
            Err(OsError {
                code: code.into(),
                caller: Location::caller(),
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
            Self::Skyline { kind } => write!(f, "{:?}", kind),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Os(os_err) => write!(f, "{}", os_err),
            Self::Skyline { kind } => write!(f, "{:?}", kind),
        }
    }
}

pub fn show_error(code: u32, message: &str, details: &str) {
    let mut message_bytes = String::from(message).into_bytes();
    let mut details_bytes = String::from(details).into_bytes();

    if message_bytes.len() > 2048 {
        message_bytes.truncate(2044);
        message_bytes.append(&mut String::from("...\0").into_bytes());
    }
    if details_bytes.len() > 2048 {
        details_bytes.truncate(2044);
        details_bytes.append(&mut String::from("...\0").into_bytes());
    }
    unsafe {
        let error = nn::err::ApplicationErrorArg::new_with_messages(
            code,
            c_str(str::from_utf8(&message_bytes).unwrap()),
            c_str(str::from_utf8(&details_bytes).unwrap()),
            &nn::settings::LanguageCode_Make(nn::settings::Language_Language_English),
        );

        nn::err::ShowApplicationError(&error);
    };
}