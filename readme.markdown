# Bottle

A Rush script to compress and encrypt (and decrypt and extract) files or directories using [age](https://github.com/FiloSottile/age) and tar. 

Bottle has no config options and only takes a single parameter, in an attempt to follow age's philosophy of simplicity.

**This program is currently just a toy. I would not use it for real-world encryption at this time.**

## Install

### Preqrequisites

1. [Install Rust](https://www.rust-lang.org/tools/install), if you haven't already. Recommend version 1.57+.
2. [Install age](https://github.com/FiloSottile/age#installation). Bottle requires age version 1.0+. The related age-keygen should be included with that install (check with age-keygen --version).

### Installing Bottle itself

1. Run `cargo install --git https://github.com/sts10/bottle-rs --branch main`
2. Make an age key-pair file for Bottle to use by running: `mkdir ~/.bottle && age-keygen -o ~/.bottle/bottle_key.txt`. You can also move a previously-created age key-pair to that location.

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
    <TARGET>    Target file or directory to either encrypt or decrypt
```

## To do

- [ ] Ability to encrypt a directory with only access to a public key. (Looks like I would use age's `-R` flag.)
- [ ] Have it be way more cautious when potentially overwriting a file or directory.
- [ ] Ability to print (public) key of key-pair at `~/.bottle/bottle_key.txt`
- [ ] Consider an option NOT to compress directory before encrypting it. Would need to be able to unbottle .tar.age files.
- [ ] An option to use your ssh key instead ([which age supports](https://github.com/FiloSottile/age#ssh-keys))
