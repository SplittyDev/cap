# Crates Package Manager
> A global package manager built on top of cargo and crates.io.

This is a work in progress.

## Installation

```
cargo install capm --locked
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
