{{LSP_FEATURES}}

![{{LSP_DEMO_ALT}}](./lsp-demo.gif)

## Installation

[Install `jjp`](../../readme.md#installation), then configure based on your editor

### VSCode

[VSCode extension](https://marketplace.visualstudio.com/items?itemName=20jasper.jjpwrgem-vscode)

### Generic LSP Client

Run `jjp lsp` as the language server for JSON files. See the [Helix](#helix) section for a full example configuration.

{{SETTINGS_REFERENCE}}

### Helix

```toml
# languages.toml
[language-server.jjp]
command = "jjp"
args = ["lsp"]

{{HELIX_FORMAT_CONFIG}}

[[language]]
name = "json"
language-servers = ["jjp"]
```
