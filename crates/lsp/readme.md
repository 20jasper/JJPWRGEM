LSP providing

- diagnostics
- code actions
- formatting

Scales well for large files. There is no perceivable delay when editing a 68k line, 5MB file. Diagnostics, code actions, and formatting take less than 20ms

![animation of JJPWRGEM's LSP. file changes are made quickly and feedback is shown quickly. code actions fix common issues like missing colons](./lsp-demo.gif)

## quick start

[Install `jjp`](../../readme.md#installation)

Configure your lsp client of choice to run `jjp lsp`

For example, here's a Helix configuration

```toml
# languages.toml
[language-server.jjp]
command = "jjp"
args = ["lsp"]

[[language]]
name = "json"
language-servers = ["jjp"]
```
