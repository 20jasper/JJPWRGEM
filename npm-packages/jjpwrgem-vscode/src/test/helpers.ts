import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

export async function activateExtension(): Promise<
  vscode.Extension<{ whenReady: Promise<void> }>
> {
  const extension = vscode.extensions.getExtension<{
    whenReady: Promise<void>;
  }>("20jasper.jjpwrgem-vscode")!;
  await extension.activate();
  await extension.exports.whenReady;
  return extension;
}

export async function withTempFile<T>(
  name: string,
  content: string,
  fn: (uri: vscode.Uri) => Promise<T>,
): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-smoke-"));
  try {
    const file = path.join(dir, name);
    fs.writeFileSync(file, content, "utf8");
    return await fn(vscode.Uri.file(file));
  } finally {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

export async function formatDocument(
  uri: vscode.Uri,
): Promise<vscode.TextEdit[]> {
  const edits = await vscode.commands.executeCommand<
    vscode.TextEdit[] | undefined
  >("vscode.executeFormatDocumentProvider", uri, {
    tabSize: 2,
    insertSpaces: true,
  });
  return edits ?? [];
}

export function jjpwrgemDiagnostics(
  uri: vscode.Uri,
  timeoutMs = 10000,
): Promise<vscode.Diagnostic[]> {
  return new Promise((resolve, reject) => {
    const subscription = vscode.languages.onDidChangeDiagnostics(check);
    const timer = setTimeout(() => {
      subscription.dispose();
      reject(new Error("no jjpwrgem diagnostics"));
    }, timeoutMs);

    function check() {
      const found = vscode.languages
        .getDiagnostics(uri)
        .filter((d) => d.source === "jjpwrgem");
      if (found.length > 0) {
        clearTimeout(timer);
        subscription.dispose();
        resolve(found);
      }
    }
    check();
  });
}
