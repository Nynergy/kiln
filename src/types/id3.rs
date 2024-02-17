use clap::ValueEnum;
use id3::frame::{
    Comment,
    Content,
    Picture,
    PictureType,
};
use std::{
    collections::HashSet,
    io::Cursor,
};

use crate::types::kiln::{
    KilnError,
    KilnErrorKind,
    KilnResult,
};

pub type TagSet = HashSet<TagPair>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, ValueEnum)]
pub enum TagId {
    TPE1,
    TPE2,
    TALB,
    TIT2,
    TRCK,
    TYER,
    TDRC,
    TCON,
    TSRC,
    COMM,
    APIC,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TagPair {
    pub id: TagId,
    pub val: Content,
}

impl TagPair {
    pub fn from_str(id: &str, val: &str) -> KilnResult<Self> {
        let tag_pair = match id {
            "TPE1" => Self { id: TagId::TPE1, val: Content::Text(val.to_string()) },
            "TPE2" => Self { id: TagId::TPE2, val: Content::Text(val.to_string()) },
            "TALB" => Self { id: TagId::TALB, val: Content::Text(val.to_string()) },
            "TIT2" => Self { id: TagId::TIT2, val: Content::Text(val.to_string()) },
            "TRCK" => Self { id: TagId::TRCK, val: Content::Text(val.to_string()) },
            "TYER" => Self { id: TagId::TYER, val: Content::Text(val.to_string()) },
            "TDRC" => Self { id: TagId::TDRC, val: Content::Text(val.to_string()) },
            "TCON" => Self { id: TagId::TCON, val: Content::Text(val.to_string()) },
            "TSRC" => Self { id: TagId::TSRC, val: Content::Text(val.to_string()) },
            "COMM" => {
                let comment = Comment {
                    lang: String::from("eng"),
                    description: String::new(),
                    text: val.to_string(),
                };
                Self { id: TagId::COMM, val: Content::Comment(comment) }
            },
            "APIC" => {
                let image = match image::open(val) {
                    Ok(img) => img,
                    Err(e) => return Err(KilnError::new(KilnErrorKind::Image, e.to_string()))
                };

                let mut encoded_image_bytes = Cursor::new(Vec::new());
                image.write_to(&mut encoded_image_bytes, image::ImageOutputFormat::Jpeg(24)).unwrap();

                let picture = Picture {
                    mime_type: "image/jpeg".to_string(),
                    picture_type: PictureType::CoverFront,
                    description: "cover".to_string(),
                    data: encoded_image_bytes.into_inner(),
                };
                Self { id: TagId::APIC, val: Content::Picture(picture) }
            },
            _ => return Err(KilnError::new(KilnErrorKind::ID3, format!("{} is not a valid id3 tag for kiln", id))),
        };

        Ok(tag_pair)
    }

    pub fn from_str_with_content(id: &str, val: Content) -> KilnResult<Self> {
        let tag_pair = match id {
            "TPE1" => Self { id: TagId::TPE1, val },
            "TPE2" => Self { id: TagId::TPE2, val },
            "TALB" => Self { id: TagId::TALB, val },
            "TIT2" => Self { id: TagId::TIT2, val },
            "TRCK" => Self { id: TagId::TRCK, val },
            "TYER" => Self { id: TagId::TYER, val },
            "TDRC" => Self { id: TagId::TDRC, val },
            "TCON" => Self { id: TagId::TCON, val },
            "TSRC" => Self { id: TagId::TSRC, val },
            "COMM" => Self { id: TagId::COMM, val },
            "APIC" => Self { id: TagId::APIC, val },
            _ => return Err(KilnError::new(KilnErrorKind::ID3, format!("{} is not a valid id3 tag for kiln", id))),
        };

        Ok(tag_pair)
    }

    pub fn from_id(id: TagId, val: Content) -> Self {
        Self { id, val }
    }
}
