use fvm_shared::error::ExitCode;
use thiserror::Error;

mod abort {
    use crate::ActorError;
    use fvm_shared::error::ExitCode;

    #[derive(thiserror::Error, Debug)]
    #[error("abort error")]
    pub struct Abort {
        /// This ensures that this error can not be crated outside.
        _private: (),
    }

    #[cfg(feature = "fil-actor")]
    fn maybe_abort(exit_code: ExitCode, msg: Option<&str>) -> ! {
        fvm_sdk::vm::abort(exit_code as u32, msg);
    }
    #[cfg(not(feature = "fil-actor"))]
    fn maybe_abort(exit_code: ExitCode, msg: Option<&str>) -> ! {
        // TODO: maybe not panic, needs discussion what we want here
        panic!("Abort: {}: {:?}", exit_code, msg);
    }

    impl From<ActorError> for Abort {
        fn from(err: ActorError) -> Self {
            let ActorError { exit_code, msg } = err;
            maybe_abort(exit_code, Some(&msg));
        }
    }

    /// Converts a raw encoding error into an ErrSerialization.
    impl From<fvm_ipld_encoding::Error> for Abort {
        fn from(e: fvm_ipld_encoding::Error) -> Self {
            maybe_abort(ExitCode::ErrSerialization, Some(&e.to_string()));
        }
    }
}

pub use abort::Abort;

/// TODO fix error system; actor errors should be transparent to the VM.
/// The error type that gets returned by actor method calls.
#[derive(Error, Debug, Clone, PartialEq)]
#[error("ActorError(exit_code: {exit_code:?}, msg: {msg})")]
pub struct ActorError {
    /// The exit code for this invocation, must not be `0`.
    exit_code: ExitCode,
    /// Message for debugging purposes,
    msg: String,
}

impl ActorError {
    pub fn new(exit_code: ExitCode, msg: String) -> Self {
        Self { exit_code, msg }
    }

    /// Returns the exit code of the error.
    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    /// Returns true when the exit code is `Ok`.
    pub fn is_ok(&self) -> bool {
        self.exit_code == ExitCode::Ok
    }

    /// Error message of the actor error.
    pub fn msg(&self) -> &str {
        &self.msg
    }

    /// Prefix error message with a string message.
    pub fn wrap(mut self, msg: impl AsRef<str>) -> Self {
        self.msg = format!("{}: {}", msg.as_ref(), self.msg);
        self
    }
}

/// Converts a raw encoding error into an ErrSerialization.
impl From<fvm_ipld_encoding::Error> for ActorError {
    fn from(e: fvm_ipld_encoding::Error) -> Self {
        Self { exit_code: ExitCode::ErrSerialization, msg: e.to_string() }
    }
}

/// Converts an actor deletion error into an actor error with the appropriate exit code. This
/// facilitates propagation.
#[cfg(feature = "fil-actor")]
impl From<fvm_sdk::error::ActorDeleteError> for ActorError {
    fn from(e: fvm_sdk::error::ActorDeleteError) -> Self {
        use fvm_sdk::error::ActorDeleteError::*;
        Self {
            // FIXME: These shouldn't be "system" errors, but we're trying to match existing
            // behavior here.
            exit_code: match e {
                BeneficiaryIsSelf => ExitCode::SysErrIllegalActor,
                BeneficiaryDoesNotExist => ExitCode::SysErrIllegalArgument,
            },
            msg: e.to_string(),
        }
    }
}

/// Converts a no-state error into an an actor error with the appropriate exit code (illegal actor).
/// This facilitates propagation.
#[cfg(feature = "fil-actor")]
impl From<fvm_sdk::error::NoStateError> for ActorError {
    fn from(e: fvm_sdk::error::NoStateError) -> Self {
        Self {
            // FIXME: These shouldn't be "system" errors, but we're trying to match existing
            // behavior here.
            exit_code: ExitCode::SysErrIllegalActor,
            msg: e.to_string(),
        }
    }
}

/// Performs conversions from SyscallResult, whose error type is ExitCode,
/// to ActorErrors. This facilitates propagation.
impl From<ExitCode> for ActorError {
    fn from(e: ExitCode) -> Self {
        ActorError { exit_code: e, msg: "".to_string() }
    }
}

/// Convenience macro for generating Actor Errors
#[macro_export]
macro_rules! actor_error {
    // Error with only one stringable expression
    ( $code:ident; $msg:expr ) => { $crate::ActorError::new(fvm_shared::error::ExitCode::$code, $msg.to_string()) };

    // String with positional arguments
    ( $code:ident; $msg:literal $(, $ex:expr)+ ) => {
        $crate::ActorError::new(fvm_shared::error::ExitCode::$code, format!($msg, $($ex,)*))
    };

    // Error with only one stringable expression, with comma separator
    ( $code:ident, $msg:expr ) => { $crate::actor_error!($code; $msg) };

    // String with positional arguments, with comma separator
    ( $code:ident, $msg:literal $(, $ex:expr)+ ) => {
        $crate::actor_error!($code; $msg $(, $ex)*)
    };

    ($code:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::actor_error!($code, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $code:ident, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::actor_error!($code, $msg).into());
        }
    };
    ($cond:expr, $code:ident, $err:expr $(,)?) => {
        if !$cond {
            return Err($crate::actor_error!($code, $err).into());
        }
    };
    ($cond:expr, $code:ident, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::actor_error!($code, $fmt, $($arg)*).into());
        }
    };
}

#[macro_export]
macro_rules! ensure_args {
    ($cond:expr, $msg:literal $(,)?) => {
        $crate::ensure!($cond, ErrIllegalArgument, $msg)
    };
    ($cond:expr, $err:expr $(,)?) => {
        $crate::ensure!($cond, ErrIlegalArgument, $err)
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::ensure!($cond, ErrIllegalArgument, $fmt, $($arg)*)
    };
}
