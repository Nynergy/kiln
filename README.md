# kiln

A command line utility for reading and writing id3 tags

------------------------------------------------------------------------------

## What is kiln and why does it exist?

_kiln_ is a command line utility that can do two things:
- Read id3 tags from a given set of files and print them to stdout
- Read id3 tags from a given file and write those tags to the specified files

For a while I've been trying to move away from traditional GUI id3 tag editors
and into the terminal, but up until now I was hacking away at a TUI solution
called [rime](https://github.com/Nynergy/rime). I was getting kind of
overwhelmed with that project, and then I was struck with a bout of inspiration!

At work one day I had to request some changes to a number of files, and so I
prepared some patch files detailing the changes so that our administrator could
just apply the patches and we could all get on with our work day. I've always
thought that the process of making and applying patches was so elegant and
simple in a Unix environment, just two lightweight utilities that can be very
flexible when thrown into shell scripts. Then I thought to myself, "What if we
could apply the same principles to modifying the metadata of non-text files?"
Thus, the idea for _kiln_ was born; I wanted to make a command line utility that
combined _diff_ and _patch_ into a single binary, and could be used very
powerfully in a scripting context.

## What exactly does it do?

When you invoke _kiln_ using the `list` subcommand, you must also provide it a
valid glob that will capture all the files you want to list the id3 tags for. At
the moment _kiln_ will ignore anything but mp3 files, so you don't have to worry
about globbing around files of other types. By default, if you don't provide a
glob yourself, _kiln_ will use `./*`, meaning all files in the current
directory.

The way _kiln_ lists tags is by outputting them to stdout in a format fairly
similar to a Windows .ini syntax. I chose this format for its simplicity not
only in reading as a user, but also in parsing as a text file, as we'll see
later on. We first list the glob that was used as a header, then all the tags
that remain consistent across all files in that glob. Then we list each
individual file as a header, and all the tags specific to that file. As an
example, let's run it against some music files I downloaded from [this Bandcamp
page](https://mapsoflowfidelity.bandcamp.com/album/opal-drifters).

While inside the directory with all the music files, let's invoke _kiln_ in the
following way:

```
$ kiln list "./*Opal*"
```

Take note of a couple things here.The glob is contained within quotes, because
without the quotes the shell would expand the glob before passing all the
filenames in as arguments to _kiln_. We don't want this, as _kiln_ should be
getting the glob itself, so that it can handle all the files and tags
internally. We're also specifying that we only want files with 'Opal' in the
name. Try using the default glob yourself and see what happens :)

When you run _kiln_ as we did above, you should see the following output in
stdout:

```
# All files in glob share the following tags:
[./*Opal*]
APIC = cover: Front cover (image/jpeg, 160854 bytes)
TPE1 = Maps of Low Fidelity
TYER = 2023
TPE2 = Maps of Low Fidelity
COMM = Visit https://mapsoflowfidelity.bandcamp.com
TALB = Opal Drifters

# The following file has these differing tags:
[Maps of Low Fidelity - Opal Drifters - 01 Opal Drifters.mp3]
TRCK = 1
TIT2 = Opal Drifters

# The following file has these differing tags:
[Maps of Low Fidelity - Opal Drifters - 02 Rethought Topographical.mp3]
TRCK = 2
TIT2 = Rethought Topographical

# The following file has these differing tags:
[Maps of Low Fidelity - Opal Drifters - 03 Endless Patience.mp3]
TRCK = 3
TIT2 = Endless Patience

# The following file has these differing tags:
[Maps of Low Fidelity - Opal Drifters - 04 Aimless Time.mp3]
TRCK = 4
TIT2 = Aimless Time

# The following file has these differing tags:
[Maps of Low Fidelity - Opal Drifters - 05 Halcyon Old Tongues.mp3]
TRCK = 5
TIT2 = Halcyon Old Tongues
```

Just like you would with _diff_ you can dump that output into a file. It's very
useful to do this, because just like _patch_, this is the same format that
_kiln_ will use to read id3 tags and make changes to the specified files.

You can try setting the tags to those files by running _kiln_ against that file
like so:

```
kiln set --ask MoLF.kiln
```

This is assuming that you dumped the output into a file called 'MoLF.kiln'. The
.kiln extension isn't necessary, but it makes it easier to know what the files
are for if you're storing them somewhere for any amount of time. Also note that
_kiln_ will just go ahead and attempt to set the tags right away if you don't
give it the `--ask` flag. We'll talk more about the options later.

Also keep in mind that the cover image tag (APIC) is listed out using MIME
information, as well as its size in bytes. When setting the cover image using
the 'set' subcommand, the APIC tag should be changed to the path where your
desired cover image is located, otherwise _kiln_ will complain that you gave it
a bad image. You can do that by changing the APIC line like this:

```
APIC = cover: Front cover (image/jpeg, 160854 bytes)

# Change the above line to be:

APIC = /path/to/cover.jpg
```

## How can I compile and run it?

First, you need to clone the repo:

```bash
$ git clone https://github.com/Nynergy/kiln.git
$ cd kiln
```

To build the app, do:

```bash
$ cargo build
$ cargo run <command> ...
```

To install it into your Cargo installation's install root, do:

```bash
$ cargo install --path .
```

Just be sure that your Cargo install root is in your PATH environment variable,
otherwise your shell won't know where to find the installed binary. Once it's
installed, you can run the app by simply running `kiln` with the appropriate
subcommand.

## How do I use it?

You can run `kiln --help` to see some usage information. There are two ways to
run it.

### List

```
$ kiln list --help
List tags for all selected files

Usage: kiln list [OPTIONS] [GLOB]

Arguments:
  [GLOB]  Glob string to select files/directories [default: ./*]

Options:
  -c, --no-comments  Turn off comments in the output
  -f, --force-empty  Force listing files with no tags
  -h, --help         Print help
```

The lines of the output that start with '#' are comments, and you can turn them
off if you don't want to see them. When parsing the resulting files for setting
tags, _kiln_ will ignore comments, so you don't have to worry about removing
them if you don't want to bother.

By default, _kiln_ will also not print out "empty" headers. That is, if a file
has no id3 metadata at all, we don't even print out that the file was grabbed by
the glob. If you want to force _kiln_ to print those empty headers (which can be
very useful when you want to prep an input file for fresh id3 tags), then you
can use the `--force-empty` option.

### Set

```
$ kiln set --help
Set tags given an input file

Usage: kiln set [OPTIONS] <INPUT_FILE>

Arguments:
  <INPUT_FILE>  Input file to read tags from

Options:
  -a, --ask                        Ask for user confirmation before writing tags to files
  -p, --preserve <PRESERVED_TAGS>  Specify a list of tags to preserve (will not be deleted) [possible values: tpe1, tpe2, talb, tit2, trck, tyer, tdrc, tcon, tsrc, comm, apic]
  -h, --help                       Print help
```

Like we mentioned above, if you don't tell _kiln_ to ask you for user
confirmation first, it will just go ahead and attempt to write the tags to the
files.

I also came across the issue during testing of wanting to modify tags other than
the cover image, but not being able to leave the APIC tag in the file because I
didn't have the cover image on hand. If you remove any tags from the file before
setting, _kiln_ will remove that tag from the file entirely, so it would have
removed the cover image from the mp3, even though I didn't want it to! My
solution was to allow the user to pass in a comma-delimited list of tags to
"preserve". What this means is that _kiln_ will not delete any preserved tags,
even if they are missing from the input file.

Note that the provided list of tag options also tells you what id3 tags _kiln_
currently supports. If it's not in the list, we don't mess with it. Perhaps in
the future we'll add more.

## Now what?

Use it, put the tags in the files, print 'em out. Enjoy yourself :)
