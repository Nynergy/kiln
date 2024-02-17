use nom::{
    bytes::complete::{
        is_a,
        tag,
        take_till,
        take_while,
    },
    character::complete::{
        alphanumeric0 as alphanumeric,
        char,
        space0 as space,
    },
    combinator::{
        map,
        opt,
    },
    error::{
        VerboseError,
        VerboseErrorKind,
    },
    multi::many0 as many,
    sequence::{
        delimited,
        pair,
        terminated,
        tuple,
    },
    IResult,
};

use crate::types::{
    id3::{
        TagPair,
        TagSet,
    },
    kiln::{
        KilnError,
        KilnErrorKind,
        KilnResult,
        Section,
    },
};

pub fn parse_input_file(content: &str) -> KilnResult<Vec<Section>> {
    let (remaining, parsed) = match sections(content) {
        Ok((remaining, parsed)) => (remaining, parsed),
        Err(nom::Err::Failure(e)) => {
            if !e.errors.is_empty() {
                match e.errors[0] {
                    (path, VerboseErrorKind::Context("Image")) => {
                        return Err(KilnError::new(KilnErrorKind::Image, format!("Bad image: {}", path)))
                    },
                    (tag, VerboseErrorKind::Context("ID3")) => {
                        return Err(KilnError::new(KilnErrorKind::ID3, format!("{} is not a valid id3 tag for kiln", tag)))
                    },
                    (item, VerboseErrorKind::Context(context)) => {
                        return Err(KilnError::new(KilnErrorKind::Parse, format!("{}: {}", context, item)))
                    },
                    _ => panic!("Something has gone horribly wrong here! {:?}", e.errors[0])
                }
            } else {

                return Err(KilnError::new(KilnErrorKind::Parse, "LMAO".to_string()))
            }
        },
        Err(e) => return Err(KilnError::new(KilnErrorKind::Parse, e.to_string())),
    };

    // If we still have remaining input, alert the user to this
    if remaining.len() > 0 {
        println!("WARN: Could not consume all the input! Here is what remains:");
        println!("\n{remaining}\n");
    }

    Ok(parsed)
}

fn sections(input: &str) -> IResult<&str, Vec<Section>, VerboseError<&str>> {
    many(section)(input)
}

fn section(input: &str) -> IResult<&str, Section, VerboseError<&str>> {
    map(
        pair(header, tag_set),
        |(h, ts)| Section { header: String::from(h), tag_set: ts }
    )(input)
}

fn header(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    terminated(
        delimited(char('['), take_while(|c| c != ']'), char(']')),
        opt(is_a(" \r\n")),
    )(input)
}

fn tag_set(input: &str) -> IResult<&str, TagSet, VerboseError<&str>> {
    map(
        many(tag_pair),
        |vec| vec.into_iter().collect(),
    )(input)
}

fn tag_pair(input: &str) -> IResult<&str, TagPair, VerboseError<&str>> {
    let (i, key) = alphanumeric(input)?;
    let (i, _) = tuple((opt(space), tag("="), opt(space)))(i)?;
    let (i, val) = take_till(|c| c == '\n')(i)?;
    let (i, _) = opt(is_a(" \r\n"))(i)?;

    match TagPair::from_str(key, val) {
        Ok(tag_pair) => Ok((i, tag_pair)),
        Err(e) => match e.kind {
            KilnErrorKind::Image => {
                Err(
                    nom::Err::Failure(
                        VerboseError {
                            errors: vec![(
                                val,
                                VerboseErrorKind::Context("Image"),
                            )]
                        }
                    )
                )
            },
            KilnErrorKind::ID3 => {
                Err(
                    nom::Err::Failure(
                        VerboseError {
                            errors: vec![(
                                key,
                                VerboseErrorKind::Context("ID3"),
                            )]
                        }
                    )
                )
            },
            _ => {
                Err(
                    nom::Err::Failure(
                        VerboseError {
                            errors: vec![]
                        }
                    )
                )
            },
        },
    }
}
