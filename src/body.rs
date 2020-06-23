use std::fs::File;
use std::io::Read;
use std::path::Path;

use bytes::BytesMut;

use crate::error::Result;

/// Request/Response body
#[derive(Clone, Debug)]
pub struct Body {
    kind: BodyKind
}

/// match body kind and process
#[macro_export]
macro_rules! body_kind {
    ($kind:expr, $text:ident => $process_text:block, $binary:ident  => $process_bin:block, $empty:ident => $process_ety:block) => {
        match $kind {
            BodyKind::Text($text) => $process_text
            BodyKind::Binary($binary) => $process_bin
            BodyKind::Empty =>  $process_ety
        }
    };
}

impl Body {
    /// create a empty Request body
    pub fn empty() -> Self {
        Self::new(BodyKind::Empty)
    }

    /// create the body use given `kind`
    pub fn new(kind: BodyKind) -> Self {
        Self {
            kind
        }
    }

    // that's weird right?
    // pub fn from_form() {}

    /// Create the Request Body from bytes
    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Self {
        Body::new(BodyKind::Binary(BytesMut::from(bytes.as_ref())))
    }
    /// Create the Request Body from string
    pub fn from_str(str: &str) -> Self {
        Body::new(BodyKind::Text(str.to_owned()))
    }

    /// Create the Request Body from string
    pub fn from_string(str: String) -> Self {
        Body::new(BodyKind::Text(str.to_owned()))
    }

    /// Create the Request Body from Vec
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Body::new(BodyKind::Binary(BytesMut::from(vec.as_slice())))
    }

    /// Create the Request Body from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Self::from_vec(data))
    }

    /// return the Body length
    pub fn body_length(&self) -> usize {
        body_kind!(self.kind(),
            text => {
                text.len()
            },
            binary => {
                binary.len()
            },
            empty => {
                0
            }
        )
    }
    /// return http body kind (Empty, Text or Binary)
    pub fn kind(&self) -> &BodyKind {
        &self.kind
    }

    // pub fn jsonify<'a,T: Deserialize<'a>>(&self) -> Result<T> {
    //     match self.kind() {
    //         BodyKind::Text(text) => {
    //             serde_json::from_str(text.to_owned().as_str())
    //         }
    //         BodyKind::Binary(buf) => {
    //             let buf = buf.clone();
    //             serde_json::from_slice(buf.as_ref())
    //         }
    //     }
    // }
}

/// The Http Request/Response Body Type
#[derive(Clone, Debug)]
pub enum BodyKind {
    /// the request content is text
    Text(String),
    /// the request content is binary
    Binary(BytesMut),
    /// the request content is empty
    Empty,
}
