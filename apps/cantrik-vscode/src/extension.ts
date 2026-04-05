import * as child_process from "child_process";
import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

const OUTPUT = "Cantrik";
const HUB = "https://cantrik.sangkan.dev";
const REPO = "https://github.com/sangkan-dev/cantrik";
const REGISTRY = "https://cantrik.sangkan.dev/registry";
let lspClient: LanguageClient | undefined;
let statusBar: vscode.StatusBarItem | undefined;

function cantrikExecutable(): string {
  const v = vscode.workspace.getConfiguration("cantrik").get<string>("executablePath");
  const t = v?.trim();
  return t && t.length > 0 ? t : "cantrik";
}

function cantrikOutput(): vscode.OutputChannel {
  return vscode.window.createOutputChannel(OUTPUT);
}

function runCantrikCapture(
  args: string[],
  title: string,
  options?: { cwd?: string }
): void {
  const ch = cantrikOutput();
  const bin = cantrikExecutable();
  ch.appendLine(`$ ${bin} ${args.join(" ")}`);
  try {
    const out = child_process.execFileSync(bin, args, {
      cwd: options?.cwd,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    ch.appendLine(out.trimEnd());
    ch.show(true);
    vscode.window.showInformationMessage(`${title} — see Cantrik output channel`);
  } catch (e: unknown) {
    const err = e as { stdout?: string; stderr?: string };
    const msg = err.stderr || err.stdout || String(e);
    ch.appendLine(msg);
    ch.show(true);
    vscode.window.showErrorMessage(`${title} failed — see Cantrik output channel`);
  }
}

function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

function refreshHarnessSummary(): void {
  const root = workspaceRoot();
  if (!root) {
    void vscode.window.showErrorMessage("Open a workspace folder first.");
    return;
  }
  const bin = cantrikExecutable();
  try {
    child_process.execFileSync(bin, ["status", "--write-harness-summary"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    openHarnessSummaryEditor();
    void vscode.window.showInformationMessage("Cantrik harness summary refreshed.");
  } catch (e: unknown) {
    const err = e as { stderr?: string; stdout?: string };
    const msg = err.stderr || err.stdout || String(e);
    void vscode.window.showErrorMessage(
      `Refresh harness summary failed: ${msg.slice(0, 200)}`
    );
  }
}

function openHarnessSummaryEditor(): void {
  const root = workspaceRoot();
  if (!root) {
    void vscode.window.showErrorMessage("Open a workspace folder first.");
    return;
  }
  const summaryPath = path.join(root, ".cantrik", "session-harness-summary.json");
  if (!fs.existsSync(summaryPath)) {
    void vscode.window.showWarningMessage(
      `No ${summaryPath}. Run "Cantrik: Write session harness summary" first.`
    );
    return;
  }
  const raw = fs.readFileSync(summaryPath, "utf8");
  let pretty = raw;
  try {
    pretty = JSON.stringify(JSON.parse(raw) as unknown, null, 2);
  } catch {
    /* keep raw */
  }
  const panel = vscode.window.createWebviewPanel(
    "cantrikHarnessSummary",
    "Cantrik harness summary",
    vscode.ViewColumn.One,
    { enableScripts: false }
  );
  panel.webview.html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <style>
    body { font-family: var(--vscode-font-family); font-size: 12px; color: var(--vscode-foreground); margin: 12px; }
    pre { white-space: pre-wrap; word-break: break-word; }
    code { font-size: 11px; }
  </style>
</head>
<body>
  <p><strong>session-harness-summary.json</strong> (<code>${escapeHtml(summaryPath)}</code>)</p>
  <pre>${escapeHtml(pretty)}</pre>
</body>
</html>`;
}

function refreshStatusBar(): void {
  if (!statusBar) {
    return;
  }
  const root = workspaceRoot();
  const label = root ? vscode.workspace.workspaceFolders![0].name : "no folder";
  statusBar.text = `$(hubot) Cantrik · ${label}`;
  statusBar.tooltip = "Cantrik — resolving version…";
  statusBar.show();
  const bin = cantrikExecutable();
  child_process.execFile(
    bin,
    ["--version"],
    { encoding: "utf8", timeout: 12_000 },
    (err, stdout) => {
      const ver = err ? "cantrik?" : stdout.trim().split("\n")[0] || "?";
      statusBar!.tooltip = `${ver}\nWorkspace: ${root ?? "(open a folder)"}`;
      statusBar!.text = `$(hubot) ${ver} · ${label}`;
    }
  );
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

class CantrikStatusWebviewProvider implements vscode.WebviewViewProvider {
  public static readonly viewId = "cantrikStatusWebview";

  resolveWebviewView(webviewView: vscode.WebviewView): void | Thenable<void> {
    webviewView.webview.options = { enableScripts: true };
    const refresh = (): void => {
      const root = workspaceRoot();
      if (!root) {
        webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage(
          "Open a workspace folder to run `cantrik status --json` here."
        );
        return;
      }
      try {
        const out = child_process.execFileSync(cantrikExecutable(), ["status", "--json"], {
          cwd: root,
          encoding: "utf8",
          maxBuffer: 16 * 1024 * 1024,
        });
        webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage(out.trimEnd());
      } catch (e: unknown) {
        const err = e as { stderr?: string; stdout?: string; message?: string };
        const msg = err.stderr || err.stdout || err.message || String(e);
        webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage(msg);
      }
    };
    webviewView.webview.onDidReceiveMessage(
      async (m: { type?: string; url?: string }) => {
        if (m.type === "refresh") {
          refresh();
        } else if (m.type === "openExternal" && m.url) {
          await vscode.env.openExternal(vscode.Uri.parse(m.url));
        }
      }
    );
    refresh();
  }

  private static htmlPage(body: string): string {
    const safe = escapeHtml(body);
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline';" />
  <style>
    body { font-family: var(--vscode-font-family); font-size: 12px; color: var(--vscode-foreground); margin: 8px; }
    pre { white-space: pre-wrap; word-break: break-word; background: var(--vscode-editor-inactiveSelectionBackground); padding: 8px; border-radius: 4px; }
    a { color: var(--vscode-textLink-foreground); }
    button { margin: 6px 0; cursor: pointer; }
  </style>
</head>
<body>
  <p><strong>Cantrik</strong> — hub, registry, and local <code>status --json</code>.</p>
  <p>
    <a href="#" id="hub">Hub</a> ·
    <a href="#" id="reg">Registry</a> ·
    <a href="#" id="repo">GitHub</a>
  </p>
  <button id="btn">Refresh status (--json)</button>
  <pre id="out">${safe}</pre>
  <script>
    const vscode = acquireVsCodeApi();
    document.getElementById('btn').onclick = () => vscode.postMessage({ type: 'refresh' });
    function openUrl(u) { vscode.postMessage({ type: 'openExternal', url: u }); }
    document.getElementById('hub').onclick = (e) => { e.preventDefault(); openUrl('${HUB}'); };
    document.getElementById('reg').onclick = (e) => { e.preventDefault(); openUrl('${REGISTRY}'); };
    document.getElementById('repo').onclick = (e) => { e.preventDefault(); openUrl('${REPO}'); };
  </script>
</body>
</html>`;
  }
}

type TreeCmd = { label: string; command: string; args?: string[] };

const TREE_ITEMS: TreeCmd[] = [
  { label: "Doctor", command: "cantrik.doctor" },
  { label: "Doctor (workspace cwd)", command: "cantrik.runInWorkspace" },
  {
    label: "Write harness summary",
    command: "cantrik.writeHarnessSummary",
  },
  {
    label: "Open harness summary (webview)",
    command: "cantrik.openHarnessSummary",
  },
  {
    label: "Refresh harness summary (write + webview)",
    command: "cantrik.refreshHarnessSummary",
  },
  { label: "Health (audit only)", command: "cantrik.health" },
  { label: "Version", command: "cantrik.version" },
  { label: "Start LSP", command: "cantrik.startLsp" },
  { label: "Stop LSP", command: "cantrik.stopLsp" },
  { label: "Open hub", command: "cantrik.openHub" },
  { label: "Open GitHub", command: "cantrik.openRepo" },
];

class CantrikTreeProvider implements vscode.TreeDataProvider<TreeCmd> {
  getTreeItem(element: TreeCmd): vscode.TreeItem {
    const item = new vscode.TreeItem(
      element.label,
      vscode.TreeItemCollapsibleState.None
    );
    item.command = {
      command: element.command,
      title: element.label,
      arguments: element.args,
    };
    return item;
  }

  getChildren(): TreeCmd[] {
    return TREE_ITEMS;
  }

  getParent(): null {
    return null;
  }
}

export function activate(context: vscode.ExtensionContext): void {
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.command = "cantrik.showOutput";
  context.subscriptions.push(statusBar);
  refreshStatusBar();
  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(() => refreshStatusBar())
  );

  const webviewProvider = new CantrikStatusWebviewProvider();
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      CantrikStatusWebviewProvider.viewId,
      webviewProvider
    )
  );

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider(
      "cantrikSidePanel",
      new CantrikTreeProvider()
    ),
    vscode.commands.registerCommand("cantrik.showOutput", () => {
      cantrikOutput().show(true);
    }),
    vscode.commands.registerCommand("cantrik.doctor", () => {
      const root = workspaceRoot();
      runCantrikCapture(["doctor"], "cantrik doctor", root ? { cwd: root } : undefined);
    }),
    vscode.commands.registerCommand("cantrik.runInWorkspace", () => {
      const root = workspaceRoot();
      if (!root) {
        void vscode.window.showErrorMessage("Open a workspace folder first.");
        return;
      }
      runCantrikCapture(["doctor"], "cantrik doctor (workspace)", { cwd: root });
    }),
    vscode.commands.registerCommand("cantrik.writeHarnessSummary", () => {
      const root = workspaceRoot();
      if (!root) {
        void vscode.window.showErrorMessage("Open a workspace folder first.");
        return;
      }
      runCantrikCapture(
        ["status", "--write-harness-summary"],
        "cantrik status --write-harness-summary",
        { cwd: root }
      );
    }),
    vscode.commands.registerCommand("cantrik.openHarnessSummary", () => {
      openHarnessSummaryEditor();
    }),
    vscode.commands.registerCommand("cantrik.refreshHarnessSummary", () => {
      refreshHarnessSummary();
    }),
    vscode.commands.registerCommand("cantrik.health", () => {
      const root = workspaceRoot();
      runCantrikCapture(
        ["health", "--no-clippy", "--no-test"],
        "cantrik health",
        root ? { cwd: root } : undefined
      );
    }),
    vscode.commands.registerCommand("cantrik.version", () => {
      runCantrikCapture(["--version"], "cantrik --version");
    }),
    vscode.commands.registerCommand("cantrik.startLsp", async () => {
      if (lspClient) {
        await lspClient.stop();
        lspClient = undefined;
      }
      const serverOptions: ServerOptions = {
        command: cantrikExecutable(),
        args: ["lsp"],
        transport: TransportKind.stdio,
      };
      const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "rust" }],
      };
      lspClient = new LanguageClient(
        "cantrikLsp",
        "Cantrik LSP",
        serverOptions,
        clientOptions
      );
      await lspClient.start();
      vscode.window.showInformationMessage("Cantrik LSP started (Rust documents).");
    }),
    vscode.commands.registerCommand("cantrik.stopLsp", async () => {
      if (lspClient) {
        await lspClient.stop();
        lspClient = undefined;
        vscode.window.showInformationMessage("Cantrik LSP stopped.");
      }
    }),
    vscode.commands.registerCommand("cantrik.openHub", async () => {
      await vscode.env.openExternal(vscode.Uri.parse(HUB));
    }),
    vscode.commands.registerCommand("cantrik.openRepo", async () => {
      await vscode.env.openExternal(vscode.Uri.parse(REPO));
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  return lspClient?.stop();
}
