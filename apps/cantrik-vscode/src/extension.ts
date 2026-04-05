import * as child_process from "child_process";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

const OUTPUT = "Cantrik";
let lspClient: LanguageClient | undefined;

function cantrikOutput(): vscode.OutputChannel {
  return vscode.window.createOutputChannel(OUTPUT);
}

function runCantrikCapture(args: string[], title: string): void {
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
  } catch (e: unknown) {
    const err = e as { stdout?: string; stderr?: string };
    const msg = err.stderr || err.stdout || String(e);
    ch.appendLine(msg);
    ch.show(true);
    vscode.window.showErrorMessage(`${title} failed — see Cantrik output channel`);
  }
}

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("cantrik.doctor", () => {
      runCantrikCapture(["doctor"], "cantrik doctor");
    }),
    vscode.commands.registerCommand("cantrik.health", () => {
      runCantrikCapture(["health", "--no-clippy", "--no-test"], "cantrik health");
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
        command: "cantrik",
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
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  return lspClient?.stop();
}
