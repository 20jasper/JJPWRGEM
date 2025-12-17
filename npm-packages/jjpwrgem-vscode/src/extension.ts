import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("jjp");

  const provider = vscode.languages.registerDocumentFormattingEditProvider(
    "json",
    {
      async provideDocumentFormattingEdits(
        document: vscode.TextDocument,
      ): Promise<vscode.TextEdit[]> {
        const fullRange = new vscode.Range(
          document.positionAt(0),
          document.positionAt(document.getText().length),
        );

        const { stderr, stdout, code } = await execJjp(document.getText());
        console.log({ stdout, stderr, code });

        if (code === 0) {
          return [vscode.TextEdit.replace(fullRange, stdout)];
        }

        output.append(stderr);

        vscode.window
          .showErrorMessage("Uh oh, your JSON is brokey 😞", "Open Logs")
          .then((choice) => {
            if (choice === "Open Logs") {
              output.show(true);
            }
          });

        return [];
      },
    },
  );

  context.subscriptions.push(provider, output);
}

export function deactivate() {}

async function execJjp(input: string) {
  return runCmdWithStdin("jjp", ["format"], input);
}

import { spawn, type SpawnOptions } from "child_process";

async function runCmdWithStdin(
  cmd: string,
  args: string[] = [],
  input: string,
  opts: SpawnOptions = {},
) {
  return new Promise<{
    stdout: string;
    stderr: string;
    code: number | null;
    signal: NodeJS.Signals | null;
  }>((resolve, reject) => {
    const child = spawn(cmd, args, {
      ...opts,
      shell: false,
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdin.write(input);
    child.stdin.end();

    child.stdout?.on("data", (d) => {
      stdout += d.toString();
    });
    child.stderr?.on("data", (d) => {
      stderr += d.toString();
    });

    child.on("error", reject);
    child.on("close", (code, signal) =>
      resolve({ stdout, stderr, code, signal }),
    );
  });
}
