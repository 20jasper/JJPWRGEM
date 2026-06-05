<!-- GENERATED FILE - update the templates in the xtask -->

# JJPWRGEM

JSON language server with rich error messages

[16–56× faster and uses 6–10× less RAM](https://github.com/20jasper/JJPWRGEM/blob/main/benches/lsp/README.md) than VS Code's built-in JSON LSP

## Requirements

Install `jjp`

```bash
mise use -g github:20jasper/jjpwrgem
```

See [releases](https://github.com/20jasper/JJPWRGEM/releases) for shell and PowerShell installation scripts, or
`npm install -g jjpwrgem`

## Features

LSP providing

- diagnostics
- code actions
- formatting

Scales well for large files. There is no perceivable delay when editing a 68k line, 5MB file. Diagnostics, code actions, and formatting take less than 20ms

![animation of JJPWRGEM's LSP. file changes are made quickly and feedback is shown quickly. code actions fix common issues like missing colons](./vscode-jjp.gif)

```
$ echo -en "{\"coolKey\"}" | jjp check
error: expected colon after key, found `}`
 --> stdin:1:11
  |
1 | {"coolKey"}
  |  ---------^
  |  |
  |  expected due to `"coolKey"`
  |
help: insert colon and placeholder value
  |
1 | {"coolKey": "🐟🛹"}
  |           ++++++++

```
