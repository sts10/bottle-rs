# Bottle

A Rush command-line tool to compress and encrypt (and decrypt and extract) files or directories using [age](https://github.com/FiloSottile/age), gzip, and tar. 

Bottle has no config options, no flags, and only takes a single parameter, in an attempt to follow age's philosophy of simplicity.

**This program is currently just a toy. I would not use it for real-world encryption/archiving at this time.** As [the age crate, which this tool uses, warns](https://docs.rs/age/0.7.1/age/index.html), "Caution: all crate versions prior to 1.0 are beta releases for testing purposes only."

## Install

### Prerequisites

1. [Install Rust](https://www.rust-lang.org/tools/install), if you haven't already. Recommend version 1.57+.
2. [Install age](https://github.com/FiloSottile/age#installation). Bottle requires age version 1.0+. The related `age-keygen` tool should be included with that install (check with `age-keygen --version` -- should also be 1.0+).

### Installing Bottle itself

1. Install Bottle by running: `cargo install --git https://github.com/sts10/bottle-rs --branch main`
2. Make an age key-pair file for Bottle to use by running: `mkdir ~/.bottle && age-keygen -o ~/.bottle/bottle_key.txt`. You can also move or copy a previously-created age key-pair to that location.

Bottle's executable command is `bottle`.

## Usage 

Bottle will always create the outputted file **in the current working directory**. It will be named automatically based on the inputted file.

Bottle will also only ever use the age key-pair located at `~/.bottle/bottle_key.txt`. Though note that this file can be copied to other machines.

- Encrypt a file with `bottle <path/to/file>`
- Compress and encrypt a directory with `bottle <path/to/directory>`. 
- Decrypt an age-encrypted file with `bottle <path/to/file>.age`
- Decrypt and extract a `.tar.gz.age` file with `bottle <path/to/archive>.tar.gz.age`.

### Help text

```
USAGE:
    bottle <TARGET>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <TARGET>    File or directory to either encrypt or decrypt. If given a directory, will tar, then gzip
                (compress), then encrypt, creating a file with the extension .tar.gz.age. If given a .tar.gz.age
                file, will decrypt and extract contents. All outputted files are placed in the current working
                directory
```

## To do

- [ ] Have it be way more cautious when potentially overwriting a file or directory.
- [ ] Ability to encrypt a directory with only access to a public key. (Like `age`'s `-R` flag.)
- [ ] Ability to print (public) key of key-pair at `~/.bottle/bottle_key.txt`
- [ ] Consider an option NOT to compress directory before encrypting it. Would need to be able to unbottle .tar.age files.
- [ ] Consider a flag to add a timestamp to the file name of encrypted files. May aid in overwriting issue.
- [ ] An option to use your ssh key instead ([which age supports](https://github.com/FiloSottile/age#ssh-keys))
- [ ] Might be neat if could read file from [stdin](https://doc.rust-lang.org/std/io/struct.Stdin.html) and/or output to stdout, so could be used in a shell-command chain.
