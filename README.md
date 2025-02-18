# docx2latex

A command line utility that converts docx files into latex templates.

# Usage

>It is important to note that `docx` files are "OOXML packages", and these packages are stored in a zipped (compressed) form.
One might expect, therefore, that the program accepts a `docx` file, unzips it, and then carries on with it's work.
However, as vast as it is, Rust ecosystem lacks packages that would allow for precise control of decompression process, namely, where to put the unzipped files.

>So instead **the user is tasked with unzipping the package**, and the program simply accepts the **path to the folder containing decompressed OOXML package**.


```
$ ./docx2latex.exe --help
A command line utility that converts docx files into latex templates

Usage: docx2latex.exe --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>    Input directory containing Office Open XML package obtained by unzipping target `.docx` file. User is tasked with unzipping the file manually to provide finer control over the filesystem
  -o, --output <OUTPUT>  Output directory, where the resulting latex and media files will be placed
  -h, --help             Print help
  -V, --version          Print version
```

# Example

There's a group of files in this repository that you may use for testing.
- `example.docx` is a sample file that contains most of featured types of data.
- `example.zip` was obtained via renaming `example.docx`, as in `cp example.docx example.zip`
- The folder `example` is the OOXML package extracted from `example.zip`

You may run the program using the following command (assuming you have [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed):

```
RUST_LOG=error cargo run -- --input example --output output
```

The `--` separates `cargo` arguments from the executable's arguments.
`--input example` sets the input directory to be the local folder `example`.
`--output output` sets the output director to be the local folder `output`.
If the output directory does not exist, it will be created, and if it does, it will be overwritten.

If the program encounters any errors, you will see messages explaining them, and if they are unrecoverable, the program will stop executing.

Once it's finished, you will find a `document.latex` in the `output` folder, as well as a `media` folder if there were images in the `.docx` file.

# Features

Both OOXML and LATEX are huge and detailed formats, but they serve vastly different purposes.
Because of this, there are some things that `docx` files specify that LATEX engines should ignore, and vice versa.
At the same time, features that overlap between the two formats are often structured in completely unfamiliar ways.
This goes to say, implementing a feature-full translator from `docx` to LATEX is an increadibly tedious task.
So rather than that, this project opts to demonstrate what is possible to achieve in this area with moderate effort.

This utility is a proof of concept.

## Todo list

- [x] Plain text
- [x] External links
- [x] Bookmarks
- [x] Images
- [x] Mathematical expressions
- [x] Special symbols
- [ ] Styles
- [ ] Graphics
- [ ] Tables

- [x] Logging

# Overview

The program will first check if there exists a `word/media` folder in the package.
If so, a `media` folder will be created in the output directory, and all media files will be copied there.
Then a `graphicx` LATEX package will be included and set to look for images in the mentioned folder.

Then the program will look for a `word/rel_/document.xml.rels` file.
This file is required by the `docx` schema, so if it is absent, the program will notify the user and finish with an error.

Lastly, equipped with information from the `.rels` file, the program will process a `word/document.xml` file in a streaming fashion.
Similarly, if it is absent the execution will result in an error.
While processing, the program keeps track of a virtual `stack` of xml tags, and uses this stack along with other contextual information to recognize when to print certain LATEX commands.

Here's a list of abbriviated tag names as taken from the code:

```rust
pub enum Tag {
    AGraphic,
    AGraphicData,
    ABlip { rel: String },
    PicPic,
    PicBlipFill,
    MoMathPara,
    MoMath,
    MDelim,
    MRad,
    MDeg,
    MRun,
    MText,
    MSub,
    MSup,
    MNary,
    MNaryPr,
    MChr { value: String },
    MFraction,
    MFunc,
    MFName,
    MNum,
    MDen,
    WPInline,
    WPAnchor,
    WBookmarkStart { anchor: String },
    WBookmarkEnd,
    WDocument,
    WDrawing,
    WParagraph,
    WRun,
    WText,
    WHyperlink(Link),
    Content(String),
    Unknown { id: String },
}
```

There are lots more tags present in OOXML packages, and most of them contain many attributes, but for the purposes of this project they have been ignored.
Only the necessary data is collected.

You might also be aware that an XML tag is not just a prefix and a name, such as "m:oMathPara".
Every tag has a 'schema' that it follows, a technical description used by OOXML processors.
Thankfully, it's possible to avoid the issue of schemas in the scope of this project.

# A note on hyperlinks

There are a few different ways that Word works with hyperlinks. According to this piece of [documentation](http://officeopenxml.com/WPhyperlink.php), OOXML uses relationship ids for external resources, and anchors for internal links. This way, information necessary for parsing the hyperlink is gathered in one place. However, the version of Office installed on my machine in particular, uses scripts to display hyperlinks when tasked to do so "in place". Scripts are beyond the scope of this project, so we will ignore this usecase for now.

# Logs

There are a number of logging messages issued by the program.
The messages are emitted through a package called [`pretty_env_logger`](https://crates.io/crates/pretty_env_logger).
Please refer to it's documentation for more details on how to use it.
To access the logs, you must set the `RUST_LOG` environment value before execution.
In `bash`, this is trivially accomplished in the following way:

```
$ RUST_LOG=info ./docx2latex.exe -i ~/Desktop/cp -o ~/Desktop/test
```

This will print all messages with importance of at least "info" to cerr.
List of message tags:

1. Error
2. Warn
3. Info
4. Debug
5. Trace

Setting `RUST_LOG` to #4 "Debug", will issue messages of levels 1 through 4, etc.

# My thanks to

[Office Open XML](http://officeopenxml.com/) is a wonderful website that explains how most things in OOXML packages work.
It has been instrumental in understanding document structure and design requirments of certain tags.

[Datypic XML vocabulary](https://www.datypic.com/sc/ooxml/) lists every tag used in OOXML packages.
While the website lacks any textual description of the tags it lists, it is easy to navigate and provides great insight into technical properties of tags, such as their expected sequencing and acceptable data types.

[Overleaf](https://www.overleaf.com) is an online platform that allows to work with LATEX files from the browser.
The infrastructure required to run LATEX on a home PC takes a lot of time and memory space to setup, so having pretty much all LATEX functionality you may need accessible from the browser is absolutely wonderful.
Overleaf team have also relased many tutorials about using LATEX, within their toolset and otherwise.

[Rust XML](https://crates.io/crates/xml) — a Rust crate for parsing XML in streaming fashion.

[Clap](https://crates.io/crates/clap) - a Rust crate for creating CLIs.

[Log](https://crates.io/crates/log) and [Pretty Env Logger](https://crates.io/crates/pretty_env_logger) for providing a simple framework for organizing logging messages.