# `kittles`

Ever wanted to open a new adjacent kitty tab? Run a command in the next available tab to the right with an equivalent cwd to the currently focused tab?

`kittles` makes extensive use of `kitty`'s kitten api to make the cli commands you always wanted but didn't have time to make.

```txt
Usage: kittles [OPTIONS]

Options:
  -d, --dont-take-focus    whether or not to focus the new tab
  -j, --jump-back          (wip) whether or not to return to original tab after command is run. currently replicates dont_take_focus behavior
  -a, --adjacent           whether or not to open tab to the right next to the currently focused tab (if applicable)
  -c, --command <COMMAND>  command, if any, to run in the new tab
  -h, --help               Print help
  -V, --version            Print version
```

## currently supported terminals & shells

terminals
- kitty

shells
- zsh
- bash
- fish
- ksh

## example `nvim` integration

```lua
-- in some lua file
vim.keymap.set('n', '67', function () vim.fn.system("kittles --adjacent --command='cargo run'") end)
vim.keymap.set('n', '45', function () vim.fn.system("kittles --dont-take-focus --adjacent --command='cargo run'") end)

```

## setup

Ensure `kitty` is installed. 

Enable [remote control](https://sw.kovidgoyal.net/kitty/remote-control/) for `kitty`:

i.e., either have the following in your `kitty.conf`:

```conf
allow_remote_control yes
```

or start `kitty` with:

```shell
kitty -o allow_remote_control=yes
```

then install this cli command

```shell
git clone https://github.com/SpicyRicecaker/kittles.git
cd kittles
cargo install --path .
# now run this through your shell or favorite editor: vim, nvim, etc.
kittles
```
