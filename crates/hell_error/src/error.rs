use core::fmt;
use std::{result, error, process, sync};



pub type HellResult<T> = result::Result<T, HellError>;
pub type HellErrorRef = sync::Arc<dyn error::Error>;



// ----------------------------------------------------------------------------
// hell error
// ----------------------------------------------------------------------------

/// we need to wrap the acutal error struct into another struct, so we can implement From<E> for any Error (including HellError)
/// using a !not Trait implementation would be nicer
#[derive(fmt::Debug)]
pub struct HellError {
    #[allow(dead_code)]
    inner: InnerHellError,
}

impl HellError {
    pub fn new(kind: HellErrorKind, content: HellErrorContent) -> Self {
        Self {
            inner: InnerHellError { kind, content }
        }
    }

    pub fn from_source<E>(kind: HellErrorKind, source: E) -> Self
    where E: error::Error + 'static
    {
        Self::new(kind, HellErrorContent::Wrapper(sync::Arc::new(source)))
    }

    pub fn from_msg(kind: HellErrorKind, msg: String) -> Self {
        Self::new(kind, HellErrorContent::Message(msg))
    }
}

impl<E> From<E> for HellError where E: error::Error + 'static {
    fn from(err : E) -> HellError {
        HellError::new(
            HellErrorKind::GenericError,
            HellErrorContent::Wrapper(sync::Arc::new(err))
        )
    }
}



// ----------------------------------------------------------------------------
// inner-hell-error
// ----------------------------------------------------------------------------

#[derive(fmt::Debug)]
pub enum HellErrorKind  {
    GenericError,
    WindowError,
    RenderError,
    ResourceError,
}

#[derive(fmt::Debug)]
pub enum HellErrorContent {
    Empty,
    Wrapper(HellErrorRef),
    Message(String),
    Code(u32)
}

#[derive(fmt::Debug)]
pub struct InnerHellError {
    kind: HellErrorKind,
    content: HellErrorContent,
}


impl InnerHellError {
    #[allow(dead_code)]
    pub fn new(kind: HellErrorKind, content: HellErrorContent) -> Self {
        Self { kind, content, }
    }

    pub fn kind(&self) -> &HellErrorKind {
        &self.kind
    }

    pub fn content(&self) -> &HellErrorContent {
        &self.content
    }
}

impl fmt::Display for InnerHellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self.content() {
            HellErrorContent::Empty        => write!(f, "[{:?}] error has no content", self.kind()),
            HellErrorContent::Wrapper(err) => err.fmt(f),
            HellErrorContent::Message(msg) => write!(f, "[{:?}] {}", self.kind(), msg),
            HellErrorContent::Code(code)   => write!(f, "[{:?}] Error-Code: '{}'", self.kind(), code)
        }
    }
}

impl error::Error for InnerHellError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.content() {
            HellErrorContent::Wrapper(err) => Some(err.as_ref()),
            _ => None
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl process::Termination for InnerHellError {
    fn report(self) -> std::process::ExitCode {
        process::ExitCode::FAILURE
    }
}



// ----------------------------------------------------------------------------
//  err-to-hell-err
// ----------------------------------------------------------------------------

pub trait ErrToHellErr<V, E>
where E: error::Error
{
    fn to_hell_err(self, kind: HellErrorKind) -> Result<V, HellError>;
    fn to_generic_hell_err(self) -> Result<V, HellError>;
    fn to_render_hell_err(self) -> Result<V, HellError>;
}

impl<V, E> ErrToHellErr<V, E> for Result<V, E>
where
    E: error::Error + 'static
{
    fn to_hell_err(self, kind: HellErrorKind) -> Result<V, HellError> {
        self.map_err(|e| {
            HellError::from_source(kind, e)
        })
    }

    fn to_generic_hell_err(self) -> Result<V, HellError> {
        self.map_err(|e| {
            HellError::from_source(HellErrorKind::GenericError, e)
        })
    }

    fn to_render_hell_err(self) -> Result<V, HellError> {
        self.map_err(|e| {
            HellError::from_source(HellErrorKind::RenderError, e)
        })
    }
}



// ----------------------------------------------------------------------------
//  err-to-hell-err
// ----------------------------------------------------------------------------

pub trait OptToHellErr<V> {
    fn to_hell_err(self, kind: HellErrorKind) -> Result<V, HellError>;
    fn to_generic_hell_err(self) -> Result<V, HellError>;
    fn to_window_hell_err(self) -> Result<V, HellError>;
    fn to_render_hell_err(self) -> Result<V, HellError>;
}

impl<V> OptToHellErr<V> for Option<V> {
    fn to_hell_err(self, kind: HellErrorKind) -> Result<V, HellError> {
        self.ok_or_else(|| HellError::from_msg(kind, "option is none".to_string()))
    }

    fn to_generic_hell_err(self) -> Result<V, HellError> {
        self.to_hell_err(HellErrorKind::GenericError)
    }

    fn to_window_hell_err(self) -> Result<V, HellError> {
        self.to_hell_err(HellErrorKind::WindowError)
    }

    fn to_render_hell_err(self) -> Result<V, HellError> {
        self.to_hell_err(HellErrorKind::RenderError)
    }
}
