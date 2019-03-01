# A Self Modifying Binary

## What is this?
This is a **prototype / experiment** for an self modifying executable. It's inspired by
[tiddlywiki](https://tiddlywiki.com/). Instead of having a separat executable
and some files containing data, all data are appended to the executable.

---

## What can it do?
  * You can dynamically add plugins. (It does IPC).
  * You can bundle executables with their dynamic libraries.

### Commands
#### Add
Adds data to the executable. Each entry has some content, an name, an some tags
with optional string values.
  * You can add files
    `./microwiki add file --name "My Cat" cat-pic.png --tag lol --tag rating=5`
  * You can add tags to existing entries.
    `./microwiki add tag "My Cat" "Cat-Pics"`
  * You can add executables or plugins.
    `./microwiki add elf ${which nano} --name nano`
    And run them via `microwiki nano`. (Dynamic libraries are bundled).

#### Web
This starts a half backed web ui for managing entries und tags. Only text
entries are supported.

#### Export
This is used for bootstrapping.
`./microwiki export -o microwiki.new && chmod +x microwiki.new` can be used
to clone the executable. Then this done, the new executable is garbage
collected. This may be necessary because data in the executable is append only.

### Special Tags
The `type`-tag is used to mark the type of an entry. The value of this tag
should be the file extension of the content. For Elf-Binaries `elf` is used.
For text `text` should be used.

When `microwiki` is run, the first argument is used to determene what
plugin/executable to run. For an entry to be selected, the `type` tag must be
`elf` and the `command` tag must equal the first command line argument.

The value of `lib` specifies a dynamic library to use. Multiple `lib` can
be used for a single entry. If none is set is `LD_LIBRARY_PATH` overwritten.

Every entry with the `web/style` is added as css when displaying an entry
via the `web` plugin. Similarly, `web/script` is included as javascript.

### Environment variables
When `OVERLAY` is provided, microwiki scans the directory for `*.entry` files
which are then loaded. _See entry files in repo._ This is used for bootstrapping.

When `OVERLAY` and `OVERLAY_WATCH` is provided, microwiki watches for changes
in the `OVERLAY_WATCH`, to reload the entry files in `OVERLAY_WATCH` and restart
the current plugin/command.

If `USE_STRIP` is set and `#cargo` is used in an entry file, the executable
gets stripped before loading.

If `USE_UPX` is set and `#cargo` is used in an entry file, the executable
gets compressed via upx before loading.

---

## Building
This runs only on linux.
Since some bootstrapping is needed, `build.sh` can be used to build the
executable. For building ~~`rust`~~ `cargo`, `clang`, `gnu-binutils` and `upx` are required.
The executable can be found at `out/microwiki`.

---

## Known Bugs
   * Web ui can't handle binary data.
   * If `OVERLAY` used the dynamic libraries in `/tmp` are not deleted after the
      programm exits.
   * If you run `strip` on `out/microwiki` all data and the main programm is
     deleted, since it's only append. Only the loader _(See loader/)_ is left.
---

## Lessons learned
  * Rust web libraries are not major enough or too complicated because of
    missing async/await.
  * Set a clear focus for a project like this.
    I'm still unsure what this is.
  * Reuse and integrate existing software.
    E.g. instead of building a web ui, a tiddywiki plugin could have been written.

### That programms like this could be useful for
  * Exporting a single executable.    Like  shipping
    scripts + payload + interpreter in one executable.
  * Building a persistent REPL.
  * Making a tiddywiki server thing like [TiddlyServer](https://github.com/Arlen22/TiddlyServer)
