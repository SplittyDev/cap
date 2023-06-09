# Rust Binary Package Manager
> A binary package manager built on top of [crates.io](https://crates.io). No 🧢.

[![asciicast](https://asciinema.org/a/RY04jcoHZ8eTCYbWJ6YL1Z2Gh.svg)](https://asciinema.org/a/RY04jcoHZ8eTCYbWJ6YL1Z2Gh)

## Why

Rust already has an amazing package manager: cargo. However, cargo is mostly a dependency manager for Rust projects. It is not primarily a binary package manager. Even though you can install binary crates with cargo, updating and maintaining them properly requires third-party solutions such as the cargo-update plugin.

This project aims to provide a simple, easy to use, and reliable binary package manager for Rust.

## Installation

```
cargo install --locked capm
```

**Or install from main branch**
```
cargo install --locked --git https://github.com/splittydev/cap
```

**Or install from source**
```
git clone https://github.com/splittydev/cap
cargo install --locked --path ./cap
```

## Usage
> Note: Not all commands are implemented yet.

**Install a package**
```
cap install <package>
```

**Uninstall a package**
```
cap uninstall <package>
```

**Update a package**
```
cap update <package>
```

**Update all packages**
```
cap update
```

**Check a package for updates**
```
cap check <package>
```

**Check all packages for updates**
```
cap check
```

**List all installed packages**
```
cap list
```

**Search for a package**
```
cap search <package>
```

**Reverse search for a binary**
```
cap search -r <binary>
```

**Show package info**
```
cap info <package>
```
