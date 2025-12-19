# `kitty-cargo-task`

Simple cli command that when run, picks the closest kitty tab to the right (or worst-case left) that has a working directory (`cwd`) identical to the current tab. If no tab is found, opens a new tab and duplicates the `cwd` of the currently focused tab. Then, runs `cargo run` on that tab.

I created this command to solve the problem of wanting to be able to "with the lowest possible friction run the current rust project, But, try to recycle terminal tabs whenever possible and keep the terminal around so I can run git and regular console commands in the new tab.".

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
vim.keymap.set('n', '67', function () vim.cmd("!kitty-cargo-task") end)
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
git clone https://github.com/SpicyRicecaker/kitty-cargo-task.git
cd kitty-cargo-task
cargo install --path .
# now run this through your shell or favorite editor: vim, nvim, etc.
kitty-cargo-task
```
