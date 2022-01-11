# Bottle

A Rush command-line tool to compress and encrypt (and decrypt and extract) files or directories using [age](https://github.com/FiloSottile/age), gzip, and tar. 

Bottle has no config options and optionally accepts two (simple) flags, in an attempt to follow age's philosophy of simplicity. It can take multiple files or directories.

**This program is currently just a toy. I would not use it for real-world encryption/archiving at this time.** As [the age crate, which this tool uses, warns](https://docs.rs/age/0.7.1/age/index.html), "Caution: all crate versions prior to 1.0 are beta releases for testing purposes only."

## Install

1. [Install Rust](https://www.rust-lang.org/tools/install), if you haven't already. Recommend version 1.57+.
2. Install Bottle by running: `cargo install --git https://github.com/sts10/bottle-rs --branch main`

Optional: While Bottle does not require [age](https://github.com/FiloSottile/age#installation) be installed, you may want to have it handy. Probably best to use age version 1.0+ when interacting with bottled files. 

Bottle's executable command is `bottle`.

## Usage 

### Things to know about Bottle

Bottle is hard-coded to use an Age Identity (basically a public/private key-pair) located at `~/.bottle/bottle_key.txt`. If there isn't a file there, Bottle will create one the first time you use Bottle.

Bottle will always create the outputted file or directory **in the current working directory**. This outputted file will be named automatically based on the inputted file. If a file or directory with that name already exists, Bottle will throw an error and quit. Users can force an overwrite with the `--force`/`-f` flag.

### The (informal) basics

- Encrypt a file with `bottle <path/to/file>`
- Compress and encrypt a directory with `bottle <path/to/directory>`
- Decrypt an age-encrypted file with `bottle <path/to/file.age>`. (Must have a `.age` extension.)
- Decrypt and extract a `.tar.gz.age` file with `bottle <path/to/archive.tar.gz.age>`. (Must have a `.tar.gz.age` extension.)

If given multiple "targets", `bottle` will act on them completely independently.
    
Note that when you encrypt a file, you **must** have access to the file at `~/.bottle/bottle_key.txt` to decrypt it at a later time. So take your bottle_key with you, but keep it safe!

### Help text

```
USAGE:
    bottle [FLAGS] [TARGETS]...

FLAGS:
    -f, --force         Force overwrite when creating a file or directory
    -h, --help          Prints help information
    -t, --time-stamp    If encrypting a file or directory, add a timestamp to the end of filename (but before file
                        extensions) of the resulting, encrypted file. Format is rfc3339, with colons replaced with
                        underscores. If decrypting a file, this flag is effectively ignored
    -V, --version       Prints version information

ARGS:
    <TARGETS>...    Files and/or directories to either encrypt or decrypt. If given a directory, will tar, then gzip
                    (compress), then encrypt, creating a file with the extension .tar.gz.age. If given a .tar.gz.age
                    file, will decrypt and extract contents. Can accept multiple targets. Bottle will act on each of
                    them separately. All outputted files are placed in the current working directory
```

## Examples

Tar, compress and encrypt a folder of your journal entries with 

```
cd ~/files-to-upload
bottle ~/Documents/journal_entries
```

This `bottle` command will create an encrypted file called `journal_entries.tar.gz.age` in `~/files-to-upload`. You can then safely upload that `journal_entries.tar.gz.age` file to a cloud service. 

Now let's say your on a new computer and want access to your journals again. To decrypt your journals to your new Documents folder, first place your bottle_key.txt file at `~/.bottle/bottle_key.txt`. Then, download the `journal_entries.tar.gz.age` file to `~/Downloads` and run:

```bash
cd ~/Documents
bottle ~/Downloads/journal_entries.tar.gz.age
```

This will place your journal_entries directory at `~/Documents/journal_entries`.

If you'd like Bottle to timestamp your encrypted files, just add a `-t` flag when you encrypt: `bottle -t ~/Documents/journal_entries`. This will create a file called `journal_entries__bottled_2022-01-10T22_49_12-05_00.tar.gz.age`. Bottle of course can decrypt this file as well: `bottle journal_entries__bottled_2022-01-10T22_49_12-05_00.tar.gz.age`.

### Multiple targets

Bottle can even encrypt multiple files and directories with one command: `bottle file1.txt directory2` will create `file1.txt.age` and `directory2.tar.gz.age`. Note that for each target given, Bottle will create completely separate encrypted files. This is different than how the `tar` command works.

## To do

- [X] Add ability to generate a key file for the user. This would eliminate the need to have age and age-keygen installed in order to use Bottle!
- [X] Have it be way more cautious when potentially overwriting a file or directory.
- [X] Consider a flag to add a timestamp to the file name of encrypted files.
- [ ] Ability to encrypt a directory with only access to a public key. (Like `age`'s `-R` flag.)
- [ ] Ability to print public key of key-pair at `~/.bottle/bottle_key.txt`
- [ ] Consider adding an option to NOT to compress directory before encrypting it. Would need to be able to unbottle .tar.age files.
- [ ] An option to use your ssh key instead ([which age supports](https://github.com/FiloSottile/age#ssh-keys))
- [ ] Might be neat if could read file from [stdin](https://doc.rust-lang.org/std/io/struct.Stdin.html) and/or output to stdout, so could be used in a shell-command chain.

## Other tools you may be interested in 

Before writing this Rust tool, I tried to do something similar using [a shell script](https://github.com/sts10/bottle). I'd say stick with this Rust version, though!
