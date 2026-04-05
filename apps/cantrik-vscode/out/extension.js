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
let lspClient;
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
function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand("cantrik.doctor", () => {
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
    }));
}
function deactivate() {
    return lspClient?.stop();
}
