{{LSP_FEATURES}}

![{{LSP_DEMO_ALT}}](./lsp-demo.gif)

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
