"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const child_process = __importStar(require("child_process"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
const OUTPUT = "Cantrik";
const HUB = "https://cantrik.sangkan.dev";
const REPO = "https://github.com/sangkan-dev/cantrik";
const REGISTRY = "https://cantrik.sangkan.dev/registry";
let lspClient;
let statusBar;
function cantrikOutput() {
    return vscode.window.createOutputChannel(OUTPUT);
}
function runCantrikCapture(args, title) {
    const ch = cantrikOutput();
    ch.appendLine(`$ cantrik ${args.join(" ")}`);
    try {
        const out = child_process.execFileSync("cantrik", args, {
            encoding: "utf8",
            maxBuffer: 8 * 1024 * 1024,
        });
        ch.appendLine(out.trimEnd());
        ch.show(true);
        vscode.window.showInformationMessage(`${title} — see Cantrik output channel`);
    }
    catch (e) {
        const err = e;
        const msg = err.stderr || err.stdout || String(e);
        ch.appendLine(msg);
        ch.show(true);
        vscode.window.showErrorMessage(`${title} failed — see Cantrik output channel`);
    }
}
function workspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
function refreshStatusBar() {
    if (!statusBar) {
        return;
    }
    const root = workspaceRoot();
    const label = root ? vscode.workspace.workspaceFolders[0].name : "no folder";
    statusBar.text = `$(hubot) Cantrik · ${label}`;
    statusBar.tooltip = "Cantrik — resolving version…";
    statusBar.show();
    child_process.execFile("cantrik", ["--version"], { encoding: "utf8", timeout: 12_000 }, (err, stdout) => {
        const ver = err ? "cantrik?" : stdout.trim().split("\n")[0] || "?";
        statusBar.tooltip = `${ver}\nWorkspace: ${root ?? "(open a folder)"}`;
        statusBar.text = `$(hubot) ${ver} · ${label}`;
    });
}
function escapeHtml(s) {
    return s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}
class CantrikStatusWebviewProvider {
    static viewId = "cantrikStatusWebview";
    resolveWebviewView(webviewView) {
        webviewView.webview.options = { enableScripts: true };
        const refresh = () => {
            const root = workspaceRoot();
            if (!root) {
                webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage("Open a workspace folder to run `cantrik status --json` here.");
                return;
            }
            try {
                const out = child_process.execFileSync("cantrik", ["status", "--json"], {
                    cwd: root,
                    encoding: "utf8",
                    maxBuffer: 16 * 1024 * 1024,
                });
                webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage(out.trimEnd());
            }
            catch (e) {
                const err = e;
                const msg = err.stderr || err.stdout || err.message || String(e);
                webviewView.webview.html = CantrikStatusWebviewProvider.htmlPage(msg);
            }
        };
        webviewView.webview.onDidReceiveMessage(async (m) => {
            if (m.type === "refresh") {
                refresh();
            }
            else if (m.type === "openExternal" && m.url) {
                await vscode.env.openExternal(vscode.Uri.parse(m.url));
            }
        });
        refresh();
    }
    static htmlPage(body) {
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
const TREE_ITEMS = [
    { label: "Doctor", command: "cantrik.doctor" },
    { label: "Health (audit only)", command: "cantrik.health" },
    { label: "Version", command: "cantrik.version" },
    { label: "Start LSP", command: "cantrik.startLsp" },
    { label: "Stop LSP", command: "cantrik.stopLsp" },
    { label: "Open hub", command: "cantrik.openHub" },
    { label: "Open GitHub", command: "cantrik.openRepo" },
];
class CantrikTreeProvider {
    getTreeItem(element) {
        const item = new vscode.TreeItem(element.label, vscode.TreeItemCollapsibleState.None);
        item.command = {
            command: element.command,
            title: element.label,
            arguments: element.args,
        };
        return item;
    }
    getChildren() {
        return TREE_ITEMS;
    }
    getParent() {
        return null;
    }
}
function activate(context) {
    statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBar.command = "cantrik.showOutput";
    context.subscriptions.push(statusBar);
    refreshStatusBar();
    context.subscriptions.push(vscode.workspace.onDidChangeWorkspaceFolders(() => refreshStatusBar()));
    const webviewProvider = new CantrikStatusWebviewProvider();
    context.subscriptions.push(vscode.window.registerWebviewViewProvider(CantrikStatusWebviewProvider.viewId, webviewProvider));
    context.subscriptions.push(vscode.window.registerTreeDataProvider("cantrikSidePanel", new CantrikTreeProvider()), vscode.commands.registerCommand("cantrik.showOutput", () => {
        cantrikOutput().show(true);
    }), vscode.commands.registerCommand("cantrik.doctor", () => {
        runCantrikCapture(["doctor"], "cantrik doctor");
    }), vscode.commands.registerCommand("cantrik.health", () => {
        runCantrikCapture(["health", "--no-clippy", "--no-test"], "cantrik health");
    }), vscode.commands.registerCommand("cantrik.version", () => {
        runCantrikCapture(["--version"], "cantrik --version");
    }), vscode.commands.registerCommand("cantrik.startLsp", async () => {
        if (lspClient) {
            await lspClient.stop();
            lspClient = undefined;
        }
        const serverOptions = {
            command: "cantrik",
            args: ["lsp"],
            transport: node_1.TransportKind.stdio,
        };
        const clientOptions = {
            documentSelector: [{ scheme: "file", language: "rust" }],
        };
        lspClient = new node_1.LanguageClient("cantrikLsp", "Cantrik LSP", serverOptions, clientOptions);
        await lspClient.start();
        vscode.window.showInformationMessage("Cantrik LSP started (Rust documents).");
    }), vscode.commands.registerCommand("cantrik.stopLsp", async () => {
        if (lspClient) {
            await lspClient.stop();
            lspClient = undefined;
            vscode.window.showInformationMessage("Cantrik LSP stopped.");
        }
    }), vscode.commands.registerCommand("cantrik.openHub", async () => {
        await vscode.env.openExternal(vscode.Uri.parse(HUB));
    }), vscode.commands.registerCommand("cantrik.openRepo", async () => {
        await vscode.env.openExternal(vscode.Uri.parse(REPO));
    }));
}
function deactivate() {
    return lspClient?.stop();
}
