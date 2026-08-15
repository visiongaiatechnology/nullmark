// STATUS: DIAMANT VGT SUPREME

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Png,
    Jpeg,
    WebP,
    Docx,
    Xlsx,
    Pptx,
    Odt,
    Svg,
    Pdf,
}

impl BinaryFormat {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::WebP => "webp",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Odt => "odt",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
        }
    }

    pub const fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Odt => "application/vnd.oasis.opendocument.text",
            Self::Svg => "image/svg+xml",
            Self::Pdf => "application/pdf",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BinaryFinding {
    pub kind: &'static str,
    pub count: usize,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBinary {
    pub format: BinaryFormat,
    pub findings: Vec<BinaryFinding>,
    pub c2pa_detected: bool,
}

impl ParsedBinary {
    pub fn metadata_count(&self) -> usize {
        self.findings.iter().map(|finding| finding.count).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedBinary {
    pub bytes: Vec<u8>,
    pub removed_items: usize,
}

pub type BinaryResult<T> = Result<T, &'static str>;
