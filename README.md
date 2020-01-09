# Godot unofficial tool

*Note: this is very much a work in progress and is subject to change*

A very basic "package" manager and project starter

```
$ gut init <project name>
```

creates a Godot project (really just adds one file right now).


# Package manager

Godot has a built in asset manager, this is more for small packages like scripts
and single scenes etc. that can be installed either from a local path or from a
remote location (e.g github.com)

## Installing a package

```
$ gut install path/to/package
```

or

```
$ gut install https://raw.githubusercontent.com/hagsteel/godot-packages/master/basic-transition/
```

or 

```
$ echo "path/to/package-a" >> requirements.txt
$ echo "path/to/package-b" >> requirements.txt
$ gut install -r requirements.txt
```

## Creating a package

To create a package, add a `manifest.toml` file to a directory containing the
files you want to include in your package.

See this example manifest:

```
name = "my-package"
author = "Hagsteel"
version = "0.3"
description = "A basic package"
usage = """
1.  Install this package with `gut`
2.  Use it
"""

files = [
    "fancy-menu.tscn",
    "fancy-menu.gd",
]
```

Only files listed under `files` in the manifest will be included when the package is installed.
