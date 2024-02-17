use glob::glob;
use id3::{
    Error,
    ErrorKind,
    Tag,
};
use std::{
    collections::HashSet,
    path::PathBuf,
};

use crate::types::{
    args::ListArgs,
    id3::{
        TagPair,
        TagSet,
    },
    kiln::{
        KilnError,
        KilnErrorKind,
        KilnResult,
    },
};

pub fn list_tags(args: ListArgs) -> KilnResult<()> {
    let glob_string = handle_glob_string(&args.glob);
    let filepaths = get_filepaths_from_glob(&glob_string)?;
    let shared_tags = construct_shared_tags(&filepaths)?;
    output_tags(&args, &filepaths, &shared_tags)?;

    Ok(())
}

// Very slipshod handling for expanding '~' in globs
fn handle_glob_string(glob_string: &str) -> String {
    match &glob_string[..1] {
        "~" => format!("{}/{}", std::env::var("HOME").unwrap(), &glob_string[1..]),
        _ => glob_string.to_string()
    }
}

fn get_filepaths_from_glob(glob_string: &str) -> KilnResult<Vec<PathBuf>> {
    let mut filepaths = Vec::new();

    let entries = match glob(glob_string) {
        Ok(entries) => entries,
        Err(e) => return Err(KilnError::new(KilnErrorKind::Glob, e.to_string())),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => return Err(KilnError::new(KilnErrorKind::Glob, e.to_string())),
        };

        if let Some(ext) = entry.extension() {
            // For now we only want to operate on mp3 files.
            if ext != "mp3" { continue; }

            filepaths.push(entry);
        }
    }

    Ok(filepaths)
}

fn construct_shared_tags(filepaths: &Vec<PathBuf>) -> KilnResult<TagSet> {
    if filepaths.is_empty() {
        return Ok(HashSet::new());
    }

    let mut tag_sets = Vec::new();
    for filepath in filepaths {
        let tag = match Tag::read_from_path(filepath) {
            Ok(tag) => tag,
            Err(Error { kind: ErrorKind::NoTag, .. }) => Tag::new(),
            Err(e) => return Err(KilnError::new(KilnErrorKind::ID3, e.to_string())),
        };

        let mut tag_set = HashSet::new();
        for frame in tag.frames() {
            let tag_pair = match TagPair::from_str_with_content(frame.id(), frame.content().clone()) {
                Ok(tag_pair) => tag_pair,
                Err(e) => return Err(e),
            };
            tag_set.insert(tag_pair);
        }

        tag_sets.push(tag_set);
    }

    let (intersection, others) = tag_sets.split_at_mut(1);
    let intersection = &mut intersection[0];
    for other in others {
        intersection.retain(|e| other.contains(e));
    }

    Ok(intersection.clone())
}

fn output_tags(args: &ListArgs, filepaths: &Vec<PathBuf>, shared_tags: &TagSet) -> KilnResult<()> {
    if shared_tags.is_empty() && !args.force_empty {
        comment(args, "# No shared tags among files in glob");
        comment(args, "");
    } else {
        comment(args, "# All files in glob share the following tags:");
        println!("[{}]", args.glob);
        for tag in shared_tags {
            println!("{:?} = {}", tag.id, tag.val);
        }
        println!("");
    }

    let mut no_tags = true;
    for filepath in filepaths {
        let tag = match Tag::read_from_path(filepath) {
            Ok(tag) => tag,
            Err(Error { kind: ErrorKind::NoTag, .. }) => Tag::new(),
            Err(e) => return Err(KilnError::new(KilnErrorKind::ID3, e.to_string())),
        };

        let mut tag_set = HashSet::new();
        for frame in tag.frames() {
            no_tags = false;
            let tag_pair = match TagPair::from_str_with_content(frame.id(), frame.content().clone()) {
                Ok(tag_pair) => tag_pair,
                Err(e) => return Err(e),
            };
            tag_set.insert(tag_pair);
        }

        let diff_tags = tag_set.difference(shared_tags)
            .collect::<HashSet<_>>();
        if !diff_tags.is_empty() || args.force_empty {
            comment(args, "# The following file has these differing tags:");
            println!("[{}]", filepath.clone().into_os_string().into_string().unwrap());
            for tag in diff_tags {
                println!("{:?} = {}", tag.id, tag.val);
            }
            println!("");
        }
    }

    if no_tags && !args.force_empty {
        comment(args, "# No tags among files in glob");
        comment(args, "");
    }

    Ok(())
}

fn comment(args: &ListArgs, string: &str) {
    if !args.no_comments {
        println!("{string}");
    }
}
