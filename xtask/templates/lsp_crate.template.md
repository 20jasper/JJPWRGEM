{{LSP_FEATURES}}

![{{LSP_DEMO_ALT}}](./lsp-demo.gif)

## Installation

[Install `jjp`](../../readme.md#installation), then configure based on your editor

### VSCode

[VSCode extension](https://marketplace.visualstudio.com/items?itemName=20jasper.jjpwrgem-vscode)

### Generic LSP Client

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
