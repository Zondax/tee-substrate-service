// https://github.com/sccommunity/rust-optee-trustzone-sdk/blob/master/optee-teec/src/error.rs
use core::fmt;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TeeErrorCode {
    /// Non-specific cause.                                                                                                       
    Generic = 0xFFFF0000,
    /// Access privileges are not sufficient.                                                                                     
    AccessDenied = 0xFFFF0001,
    /// The operation was canceled.                                                                                               
    Cancel = 0xFFFF0002,
    /// Concurrent accesses caused conflict.                                                                                      
    AccessConflict = 0xFFFF0003,
    /// Too much data for the requested operation was passed.                                                                     
    ExcessData = 0xFFFF0004,
    /// Input data was of invalid format.                                                                                         
    BadFormat = 0xFFFF0005,
    /// Input parameters were invalid.                                                                                            
    BadParameters = 0xFFFF0006,
    /// Operation is not valid in the current state.                                                                              
    BadState = 0xFFFF0007,
    /// The requested data item is not found.                                                                                     
    ItemNotFound = 0xFFFF0008,
    /// The requested operation should exist but is not yet implemented.                                                          
    NotImplemented = 0xFFFF0009,
    /// The requested operation is valid but is not supported in this implementation.                                             
    NotSupported = 0xFFFF000A,
    /// Expected data was missing.                                                                                                
    NoData = 0xFFFF000B,
    /// System ran out of resources.                                                                                              
    OutOfMemory = 0xFFFF000C,
    /// The system is busy working on something else.                                                                             
    Busy = 0xFFFF000D,
    /// Communication with a remote party failed.                                                                                 
    Communication = 0xFFFF000E,
    /// A security fault was detected.                                                                                            
    Security = 0xFFFF000F,
    /// The supplied buffer is too short for the generated output.                                                                
    ShortBuffer = 0xFFFF0010,
    /// Implementation defined error code.                                                                                        
    ExternalCancel = 0xFFFF0011,
    /// Implementation defined error code: trusted Application has panicked during the operation.                                 
    TargetDead = 0xFFFF3024,
    /// Public key type is not supported
    KeyNotSupported = 0xFFFF3025,
    /// Pair not found for public key and KeyTypeId
    PairNotFound = 0xFFFF3026,
    /// Validation error
    ValidationError = 0xFFFF3027,
    /// Keystore unavailable
    Unavailable = 0xFFFF3028,
    /// Unknown error.   
    Unknown = 0xFFFF3029,
}

impl From<u32> for TeeErrorCode {
    fn from(code: u32) -> Self {
        match code {
            0xFFFF0000..=0xFFFF0011 => unsafe { core::mem::transmute(code) },
            0xFFFF3024..=0xFFFF3028 => unsafe { core::mem::transmute(code) },
            _ => Self::Unknown,
        }
    }
}

impl TeeErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeeErrorCode::Generic => "Non-specific cause.",
            TeeErrorCode::AccessDenied => "Access privileges are not sufficient.",
            TeeErrorCode::Cancel => "The operation was canceled.",
            TeeErrorCode::AccessConflict => "Concurrent accesses caused conflict.",
            TeeErrorCode::ExcessData => "Too much data for the requested operation was passed.",
            TeeErrorCode::BadFormat => "Input data was of invalid format.",
            TeeErrorCode::BadParameters => "Input parameters were invalid.",
            TeeErrorCode::BadState => "Operation is not valid in the current state.",
            TeeErrorCode::ItemNotFound => "The requested data item is not found.",
            TeeErrorCode::NotImplemented => {
                "The requested operation should exist but is not yet implemented."
            }
            TeeErrorCode::NotSupported => {
                "The requested operation is valid but is not supported in this implementation."
            }
            TeeErrorCode::NoData => "Expected data was missing.",
            TeeErrorCode::OutOfMemory => "System ran out of resources.",
            TeeErrorCode::Busy => "The system is busy working on something else.",
            TeeErrorCode::Communication => "Communication with a remote party failed.",
            TeeErrorCode::Security => "A security fault was detected.",
            TeeErrorCode::ShortBuffer => {
                "The supplied buffer is too short for the generated output."
            }
            TeeErrorCode::ExternalCancel => "Undocumented.",
            TeeErrorCode::TargetDead => "Trusted Application has panicked during the operation.",

            TeeErrorCode::KeyNotSupported => "Key not supported",
            TeeErrorCode::PairNotFound => "Pair was not found",
            TeeErrorCode::ValidationError => "Validation error",
            TeeErrorCode::Unavailable => "Keystore unavailable",
            TeeErrorCode::Unknown => "Unknown error.",
        }
    }
}

pub struct TeeError {
    code: u32,
}

impl TeeError {
    pub fn new(kind: TeeErrorCode) -> Self {
        TeeError { code: kind as u32 }
    }

    /// Creates a new instance of an `Error` from a particular TEE error code.                                                    
    ///                                                                                                                           
    pub fn from_raw_error(code: u32) -> Self {
        Self { code }
    }

    pub fn kind(&self) -> TeeErrorCode {
        TeeErrorCode::from(self.code)
    }

    /// Returns raw code of this error.                                                                                           
    pub fn raw_code(&self) -> u32 {
        self.code
    }

    /// Returns corresponding error message of this error.                                                                        
    pub fn message(&self) -> &str {
        self.kind().as_str()
    }
}

impl fmt::Debug for TeeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} (error code 0x{:x})", self.message(), self.code)
    }
}

impl fmt::Display for TeeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} (error code 0x{:x})", self.message(), self.code)
    }
}

//#[cfg(feature = "std")]
//impl std::error::Error for TeeError {
//    fn description(&self) -> &str {
//        self.message()
//    }
//}

impl From<TeeErrorCode> for TeeError {
    #[inline]
    fn from(kind: TeeErrorCode) -> TeeError {
        TeeError { code: kind as u32 }
    }
}
