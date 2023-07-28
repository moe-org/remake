use std::error::Error;
use std::fmt;

/// A error,throw it when parse format
#[derive(Debug)]
pub struct ParseError {
    pub source: Option<&'static dyn Error>,
    pub source_span: Option<(u64, u64)>,
    pub reason: Option<String>,
}

impl ParseError {
    pub fn from_exceptional_eof() -> ParseError {
        ParseError {
            source: None,
            source_span: None,
            reason: Some(String::from(
                "Exceptional End-Of-File(or may be a byte array). The build file may be broken.",
            )),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Remake crash when parse a file").unwrap();

        match self.source {
            None => {
                write!(f, "Source Error:Unknown")
            }
            Some(err) => {
                write!(f, "Source Error:{}", err)
            }
        }
        .unwrap();

        match self.source_span {
            None => {
                write!(f, "Source Span:Unknown")
            }
            Some(err) => {
                write!(f, "Source Span:[{}..{}]", err.0, err.1)
            }
        }
        .unwrap();

        match &self.reason {
            None => {
                write!(f, "Source Reason:Unknown")
            }
            Some(err) => {
                write!(f, "Source Reason:{}", err)
            }
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
    }
}

/// An runtime error,throw it when execute.
#[derive(Debug)]
pub struct RuntimeError {
    pub source: Option<Box<dyn Error>>,
    pub command: Option<String>,
    pub reason: Option<String>,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Remake crash when parse a file").unwrap();

        match &self.source {
            None => {
                write!(f, "Source Error:Unknown")
            }
            Some(err) => {
                write!(f, "Source Error:{}", err)
            }
        }
        .unwrap();

        match &self.command {
            None => {
                write!(f, "Source Command:Unknown")
            }
            Some(err) => {
                write!(f, "Source Command:{}", err)
            }
        }
        .unwrap();

        match &self.reason {
            None => {
                write!(f, "Source Reason:Unknown")
            }
            Some(err) => {
                write!(f, "Source Reason:{}", err)
            }
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
