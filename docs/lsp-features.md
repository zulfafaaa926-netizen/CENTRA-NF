# LSP Server Features

The CENTRA-NF Language Server Protocol (LSP) implementation provides comprehensive support for code editing in compatible editors (VS Code, Vim, Neovim, etc.).

## Core Features

### 1. Document Synchronization
- **Full Document Sync** (`textDocumentSync: 1`)
- Automatic compilation on document open, change, and close
- Real-time diagnostics publishing

### 2. Diagnostics Publishing
- Real-time error reporting for syntax and semantic errors
- Severity levels: error, warning, info, hint
- Diagnostic ranges with precise line and character information
- `textDocument/publishDiagnostics` notifications

### 3. Hover Information
- **Method**: `textDocument/hover`
- Displays line content and context information
- Markdown-formatted hover text
- Returns position and contents

### 4. Code Completion
- **Method**: `textDocument/completion`
- CENTRA-NF language keywords and operations
- Completion items include:
  - Division keywords (IDENTIFICATION, ENVIRONMENT, DATA, PROCEDURE)
  - Operations (COMPRESS, VERIFY-INTEGRITY)
  - Detailed descriptions and documentation
- CompletionItem kind: 14 (Keyword), 6 (Method)

### 5. Goto Definition
- **Method**: `textDocument/definition`
- Returns definition location for symbols
- Range information showing exact location in source

### 6. Find References
- **Method**: `textDocument/references`
- Locates all occurrences of a symbol in document
- Returns array of Location objects with ranges
- Enables "Find All References" feature

### 7. Rename Refactoring
- **Method**: `textDocument/rename`
- Safe rename of symbols with multi-occurrence support
- Returns WorkspaceEdit with all text changes
- Updates all references automatically
- Supports whole-document refactoring

### 8. Document Symbols
- **Method**: `textDocument/documentSymbol`
- Lists all divisions in document
- Enables quick navigation to sections
- Symbol kind: 11 (Section)
- Full range information for each symbol

### 9. Workspace Symbols
- **Method**: `workspace/symbol`
- Global symbol search across workspace
- Case-insensitive query matching
- Returns matching keywords and predefined symbols
- Foundation for semantic symbol resolution

## Server Capabilities

The LSP server advertises these capabilities upon initialization:

```json
{
  "textDocumentSync": 1,
  "diagnosticProvider": true,
  "hoverProvider": true,
  "completionProvider": {
    "resolveProvider": false,
    "triggerCharacters": []
  },
  "definitionProvider": true,
  "referencesProvider": true,
  "renameProvider": true,
  "documentSymbolProvider": true,
  "workspaceSymbolProvider": true
}
```

## Request/Response Format

All requests follow LSP 2.0 JSON-RPC specification with Content-Length framing.

### Example: Hover Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "textDocument/hover",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.cnf"
    },
    "position": {
      "line": 0,
      "character": 5
    }
  }
}
```

### Example: Completion Request
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "textDocument/completion",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.cnf"
    },
    "position": {
      "line": 0,
      "character": 0
    }
  }
}
```

### Example: References Request
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "textDocument/references",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.cnf"
    },
    "position": {
      "line": 0,
      "character": 5
    },
    "context": {
      "includeDeclaration": true
    }
  }
}
```

### Example: Rename Request
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "textDocument/rename",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.cnf"
    },
    "position": {
      "line": 0,
      "character": 5
    },
    "newName": "NEW_IDENTIFIER"
  }
}
```

### Example: Workspace Symbol Request
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "workspace/symbol",
  "params": {
    "query": "IDENTIFICATION"
  }
}
```

## Document Lifecycle

1. Client calls `initialize` → Server returns capabilities
2. Client calls `textDocument/didOpen` → Server compiles and publishes diagnostics
3. Client calls `textDocument/didChange` → Server recompiles and publishes diagnostics
4. Client can call `textDocument/hover`, `textDocument/completion`, etc.
5. Client calls `textDocument/didClose` → Server removes document from memory
6. Client calls `shutdown` → Server prepares for exit

## Testing

All LSP features are tested with:
- 25 unit tests (protocol structure, message handling)
- 14 integration tests (end-to-end protocol validation)
- Determinism tests (same input → identical output)
- Round-trip serialization tests

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| initialization | ✅ Complete | Server capabilities advertised |
| diagnostics | ✅ Complete | Real-time error reporting |
| hover | ✅ Complete | Line content and position info |
| completion | ✅ Complete | Language keywords and operations |
| goto definition | ✅ Complete | Returns definition location |
| find references | ✅ Complete | All symbol occurrences in document |
| rename | ✅ Complete | Workspace edit with all changes |
| document symbols | ✅ Complete | Division extraction and ranges |
| workspace symbols | ✅ Complete | Global symbol search with query |
| did open/change/close | ✅ Complete | Full document synchronization |
| shutdown | ✅ Complete | Graceful server exit |

## Future Enhancements

Potential additional features:
- Signature help (`textDocument/signatureHelp`) - Parameter hints
- Call hierarchy (`textDocument/callHierarchy`) - Function call chains
- Implementation locations (`textDocument/implementation`) - Find implementations
- Semantic tokens for syntax highlighting - Advanced code coloring
- Format on save (`textDocument/formatting`) - Auto-format code
- Range formatting (`textDocument/rangeFormatting`) - Format selection
- Code lenses (`textDocument/codeLens`) - Inline metadata
- Inlay hints (`textDocument/inlayHint`) - Type annotations
- Fold ranges (`textDocument/foldingRange`) - Code folding support
- On-type formatting (`textDocument/onTypeFormatting`) - Format as you type
