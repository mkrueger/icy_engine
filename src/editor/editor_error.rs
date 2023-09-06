use std::error::Error;

#[derive(Debug, Clone)]
pub enum EditorError {
    CurrentLayerInvalid,

    Error(String),
    InvalidLayer(usize),
}

impl std::fmt::Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorError::CurrentLayerInvalid => write!(f, "Current layer is invalid"),
            EditorError::InvalidLayer(layer) => write!(f, "Layer {layer} is invalid"),
            EditorError::Error(err) => write!(f, "Editor error: {err}"),
        }
    }
}

impl Error for EditorError {
    fn description(&self) -> &str {
        "use std::display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
