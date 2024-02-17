use glob::glob;
use id3::{
    Error,
    ErrorKind,
    Frame,
    Tag,
    TagLike,
    Version,
};
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fs,
    io::{
        stdin,
        stdout,
        Write,
    },
};

use crate::{
    parse::parse_input_file,
    types::{
        args::SetArgs,
        id3::{
            TagId,
            TagPair,
            TagSet,
        },
        kiln::{
            Diff,
            FileDiff,
            KilnError,
            KilnErrorKind,
            KilnResult,
            Section,
        },
    },
};

pub fn set_tags(args: SetArgs) -> KilnResult<()> {
    let content = match fs::read_to_string(args.input_file) {
        Ok(content) => content,
        Err(e) => return Err(KilnError::new(KilnErrorKind::File, e.to_string())),
    };
    let content = remove_comments(content);
    let sections = match parse_input_file(&content) {
        Ok(sections) => sections,
        Err(e) => return Err(e),
    };

    let diff = calculate_diff(sections, args.preserved_tags)?;
    let mut no_diffs = true;
    for filediff in &diff {
        if !filediff.diffs.is_empty() {
            no_diffs = false;
            println!("{}\n", filediff);
        }
    }

    if no_diffs {
        println!("No changes to make to any files, exiting...");
        return Ok(());
    }

    if args.ask {
        match &get_user_confirmation()[..] {
            "y" | "yes" | "" => {},
            _ => {
                println!("No changes will be made to files. Exiting...");
                return Ok(());
            }
            // Anything that isn't "Yes" is "No"
        }
    }

    println!("Making changes to files...");
    commit_changes_to_files(diff)?;

    Ok(())
}

fn remove_comments(content: String) -> String {
    let mut ret = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();

    for line in lines {
        // We also remove empty lines
        if line.len() == 0 { continue; }
        if &line[0..1] != "#" {
            ret.push(line);
        }
    }

    ret.join("\n")
}

fn calculate_diff(sections: Vec<Section>, preserved_tags: Vec<TagId>) -> KilnResult<Vec<FileDiff>> {
    let old_tags = get_old_tags_from_sections(&sections)?;
    let new_tags = get_new_tags_from_sections(&sections)?;

    let mut diffs = Vec::new();

    for (header, new_set) in new_tags {
        let mut filediff = FileDiff::from(header.clone());

        // Convert our Sets into Vecs for ease of use
        let new_set = new_set.iter().collect::<Vec<_>>();
        let old_set = old_tags.get(&header).unwrap();
        let old_set = old_set.iter().collect::<Vec<_>>();

        // First we check for new and modified tags
        for new_tag in &new_set {
            if let Some(old_tag) = find_tag_by_id(&old_set, new_tag.id) {
                if old_tag.val != new_tag.val {
                    filediff.diffs.push(
                        Diff::Modify(
                            TagPair::from_id(old_tag.id, old_tag.val),
                            TagPair::from_id(new_tag.id, new_tag.val.clone())
                        )
                    );
                }
            } else {
                filediff.diffs.push(
                    Diff::Add(
                        TagPair::from_id(new_tag.id, new_tag.val.clone())
                    )
                );
            }
        }

        // Then we double back to look for deleted tags
        for old_tag in old_set {
            if let Some(_) = find_tag_by_id(&new_set, old_tag.id) {
                continue;
            } else {
                // If we want to preserve this tag, then don't even create the diff
                if preserved_tags.contains(&old_tag.id) {
                    continue;
                }
                filediff.diffs.push(
                    Diff::Delete(
                        TagPair::from_id(old_tag.id, old_tag.val.clone())
                    )
                );
            }
        }

        diffs.push(filediff);
    }

    Ok(diffs)
}

fn get_user_confirmation() -> String {
    let mut buf = String::new();
    
    print!("Allow the above changes to be written to files? [Y/n] ");

    let _ = stdout().flush();
    stdin().read_line(&mut buf).expect("Did not enter a valid string");

    // Remove line breaks
    if let Some('\n') = buf.chars().next_back() {
        buf.pop();
    }
    if let Some('\r') = buf.chars().next_back() {
        buf.pop();
    }

    buf.to_lowercase()
}

fn commit_changes_to_files(diff: Vec<FileDiff>) -> KilnResult<()> {
    for file in diff {
        write_changes_to_file(file)?;
    }

    Ok(())
}

fn get_old_tags_from_sections(sections: &Vec<Section>) -> KilnResult<HashMap<String, TagSet>> {
    let mut tag_map = HashMap::new();

    for section in sections {
        let entries = match glob(&section.header) {
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

                let path_string = entry.clone().into_os_string().into_string().unwrap();

                // If we've already added this to the map, ignore it
                if let Some(_) = tag_map.get(&path_string) { continue; }

                let tag = match Tag::read_from_path(entry) {
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

                tag_map.insert(path_string, tag_set);
            }
        }
    }

    Ok(tag_map)
}

fn get_new_tags_from_sections(sections: &Vec<Section>) -> KilnResult<HashMap<String, TagSet>> {
    let mut tag_map: HashMap<String, TagSet> = HashMap::new();

    for section in sections {
        let entries = match glob(&section.header) {
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

                let path_string = entry.clone().into_os_string().into_string().unwrap();

                // If we've already added this, then append to the tag_set
                let mut new_tag_set = section.tag_set.clone();
                if let Some(tag_set) = tag_map.get(&path_string) {
                    new_tag_set.extend(tag_set.clone());
                }
                tag_map.insert(path_string, new_tag_set);
            }
        }
    }

    Ok(tag_map)
}

fn find_tag_by_id(vec: &Vec<&TagPair>, id: TagId) -> Option<TagPair> {
    for tag in vec {
        if tag.id == id {
            return Some((*tag).clone());
        }
    }

    None
}

fn write_changes_to_file(filediff: FileDiff) -> KilnResult<()> {
    let mut tag = match Tag::read_from_path(&filediff.filepath) {
        Ok(tag) => tag,
        Err(Error { kind: ErrorKind::NoTag, .. }) => Tag::new(),
        Err(e) => return Err(KilnError::new(KilnErrorKind::ID3, e.to_string())),
    };

    for change in filediff.diffs {
        match change {
            Diff::Add(tag_pair) => {
                tag.add_frame(
                    Frame::with_content(
                        format!("{:?}", tag_pair.id),
                        tag_pair.val
                    )
                );
            },
            Diff::Delete(tag_pair) => {
                tag.remove(format!("{:?}", tag_pair.id));
            },
            Diff::Modify(_, tag_pair) => {
                tag.add_frame(
                    Frame::with_content(
                        format!("{:?}", tag_pair.id),
                        tag_pair.val
                    )
                );
            }
        }
    }

    println!("Writing changes to file {:?} ...", filediff.filepath);
    match tag.write_to_path(filediff.filepath, Version::Id3v24) {
        Ok(_) => Ok(()),
        Err(e) => Err(KilnError::new(KilnErrorKind::ID3, e.to_string())),
    }
}
