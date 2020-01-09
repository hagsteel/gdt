# Godot unofficial tool

A very basic "package" manager and project starter

Running

```
$ gut init <project name>
```

creates a Godot project.


# "Package manager"

Godot has a built in asset manager, this is more for small packages like scripts
and single scenes etc.

## Installing a package



## Manifest file

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
