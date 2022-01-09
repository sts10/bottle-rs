# Bottle

A Rush script to compress and encrypt (and decrypt and extract) files or directories using [age](https://github.com/FiloSottile/age) and tar. 

Bottle has no config options and only takes a single parameter, in an attempt to follow age's philosophy of simplicity.

## Install

1. [Install Rust](https://www.rust-lang.org/tools/install), if you haven't already. Recommend version 1.57+.
2. Run `cargo install --git https://github.com/sts10/bottle-rs --branch main`
3. Make an age key for Bottle to use by running: `mkdir ~/.bottle && age-keygen -o ~/.bottle/bottle.key`

## Usage 

Bottle will always create the outputted file **in the current working directory**. It will be named automatically based on the inputted file.

- Encrypt a file with `bottle <path/to/file>`
- Compress and encrypt a directory with `bottle <path/to/directory>`. 
- Decrypt an age-encrypted file with `bottle <path/to/file>.age`
- Decrypt and extract a `.tar.gz.age` file with `bottle <path/to/archive>.tar.gz.age`.

## To do

- [ ] Ability to encrypt a directory with only access to a public key. (Looks like I would use age's `-R` flag.)
- [ ] Ability to print (public) key of key-pair at `~/age/bottle.key`
- [ ] Consider an option NOT to compress directory before encrypting it. Would need to be able to unbottle .tar.age files.
- [ ] An option to use your ssh key instead ([which age supports](https://github.com/FiloSottile/age#ssh-keys))
