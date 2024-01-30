# Pagwin's Project Initializer (ppi)

this is a program I've written for easy initialization of projects via a cli tool. The tool is configured via a `~/.config/ppi/config.toml` file, an example file can be found in this repo (skeletons work the same way scripts do but you give a git url instead of a script).

Any project that can be initialized from a static git repo or an executable script can be easily initialized with this program.

## Packaging

The PKGBUILD file should provide all information to build this as I tested the build in a fresh chroot barring the warning down below.

**WARNING** making the PKGBUILD was very annoying due to copious amounts of linker errors caused by my usage of libgit2 and the resulting package was not tested in a clean environment. As such if you get linker errors when you go to build this I will refer you to the "WITHOUT WARRANTY" section of the MIT license and if you find any missing runtime dependencies feel free to make a PR or issue (PR being preferred over issue).
