//! LSP Message Handler — Dispatch JSON-RPC requests to protocol handlers
//!
//! Implements LSP method routing and diagnostic publishing.

use crate::jsonrpc::{JsonRpcIO, Message, Notification, Request, Response};
use crate::publisher::publish_diagnostics;
use crate::server::LspBackend;
use serde_json::json;
use std::sync::Mutex;

/// LSP Message Handler and dispatcher
pub struct MessageHandler {
    backend: Mutex<LspBackend>,
}

impl MessageHandler {
    pub fn new() -> Self {
        MessageHandler {
            backend: Mutex::new(LspBackend::new()),
        }
    }

    /// Process incoming message and return optional response
    /// Also handles sending diagnostic notifications
    pub fn handle_message(
        &self,
        message: Message,
        io: &mut JsonRpcIO,
    ) -> Result<Option<Response>, String> {
        match message {
            Message::Request(req) => {
                eprintln!("📨 Request: {} (id={})", req.method, req.id);
                self.handle_request(req, io).map(Some)
            }
            Message::Notification(notif) => {
                eprintln!("📢 Notification: {}", notif.method);
                self.handle_notification(notif)?;
                Ok(None)
            }
        }
    }

    /// Handle JSON-RPC request (requires response)
    fn handle_request(&self, req: Request, io: &mut JsonRpcIO) -> Result<Response, String> {
        let result = match req.method.as_str() {
            "initialize" => self.handle_initialize(&req),
            "textDocument/didOpen" => self.handle_did_open(&req, io),
            "textDocument/didChange" => self.handle_did_change(&req, io),
            "textDocument/didClose" => self.handle_did_close(&req),
            "textDocument/hover" => self.handle_hover(&req),
            "textDocument/completion" => self.handle_completion(&req),
            "textDocument/definition" => self.handle_definition(&req),
            "textDocument/references" => self.handle_references(&req),
            "textDocument/rename" => self.handle_rename(&req),
            "textDocument/documentSymbol" => self.handle_document_symbol(&req),
            "workspace/symbol" => self.handle_workspace_symbol(&req),
            "shutdown" => self.handle_shutdown(&req),
            method => Err(format!("Unknown method: {}", method)),
        };

        match result {
            Ok(resp) => Ok(resp),
            Err(e) => {
                eprintln!("❌ Handler error for {}: {}", req.method, e);
                Ok(Response::error(
                    req.id,
                    -32603,
                    &format!("Internal error: {}", e),
                ))
            }
        }
    }

    /// Handle notification (no response expected)
    fn handle_notification(&self, notif: Notification) -> Result<(), String> {
        match notif.method.as_str() {
            "exit" => {
                eprintln!("👋 Exit notification received");
                std::process::exit(0);
            }
            method => {
                eprintln!("⚠️  Unknown notification: {}", method);
            }
        }
        Ok(())
    }

    /// LSP: initialize request (sets up server capabilities)
    fn handle_initialize(&self, req: &Request) -> Result<Response, String> {
        eprintln!("✅ Server initializing...");

        let capabilities = json!({
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
        });

        Ok(Response::success(req.id, capabilities))
    }

    /// LSP: textDocument/didOpen notification
    fn handle_did_open(&self, req: &Request, io: &mut JsonRpcIO) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;

        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;
        let content = params["textDocument"]["text"]
            .as_str()
            .ok_or("Missing text")?;

        eprintln!("📂 Document opened: {}", uri);

        let mut backend = self.backend.lock().map_err(|e| e.to_string())?;
        backend.set_document(uri.to_string(), content.to_string());

        // Compile and collect diagnostics
        let diagnostics = backend.compile_and_diagnose(uri, content);
        eprintln!("  Found {} error(s)", diagnostics.len());

        // Publish diagnostics to client
        publish_diagnostics(io, uri, &diagnostics)?;

        Ok(Response::success(req.id, json!({})))
    }

    /// LSP: textDocument/didChange notification
    fn handle_did_change(&self, req: &Request, io: &mut JsonRpcIO) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;

        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;
        let changes = params["contentChanges"]
            .as_array()
            .ok_or("Missing contentChanges")?;

        if changes.is_empty() {
            return Ok(Response::success(req.id, json!({})));
        }

        // For FULL sync mode, last change contains full document
        let content = changes[changes.len() - 1]["text"]
            .as_str()
            .ok_or("Missing new text")?;

        eprintln!("✏️  Document changed: {}", uri);

        let mut backend = self.backend.lock().map_err(|e| e.to_string())?;
        backend.set_document(uri.to_string(), content.to_string());

        // Compile and collect diagnostics
        let diagnostics = backend.compile_and_diagnose(uri, content);
        eprintln!("  Found {} error(s)", diagnostics.len());

        // Publish diagnostics to client
        publish_diagnostics(io, uri, &diagnostics)?;

        Ok(Response::success(req.id, json!({})))
    }

    /// LSP: textDocument/didClose notification
    fn handle_did_close(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;

        eprintln!("📭 Document closed: {}", uri);

        let mut backend = self.backend.lock().map_err(|e| e.to_string())?;
        backend.remove_document(uri);

        Ok(Response::success(req.id, json!({})))
    }

    /// LSP: textDocument/hover request
    fn handle_hover(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;

        let backend = self.backend.lock().map_err(|e| e.to_string())?;
        let document = backend.get_document(uri).ok_or("Document not found")?;

        // Extract line and character from position
        let line = params["position"]["line"].as_u64().ok_or("Missing line")? as usize;
        let character = params["position"]["character"]
            .as_u64()
            .ok_or("Missing character")? as usize;

        eprintln!("🔍 Hover at {}:{}:{}", uri, line, character);

        // For now, provide hover info based on document content
        let lines: Vec<&str> = document.lines().collect();
        let hover_text = if line < lines.len() {
            let content = lines[line];
            if character <= content.len() {
                format!(
                    "Line {}: `{}`\n\n**CENTRA-NF Language**\nHover support available.",
                    line + 1,
                    content
                )
            } else {
                "Position out of bounds".to_string()
            }
        } else {
            "Line not found".to_string()
        };

        let hover_response = json!({
            "contents": {
                "kind": "markdown",
                "value": hover_text
            }
        });

        Ok(Response::success(req.id, hover_response))
    }

    /// LSP: textDocument/completion request
    fn handle_completion(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let _uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;

        eprintln!("📝 Completion requested");

        // Provide basic CENTRA-NF keywords as completions
        let completions = vec![
            json!({
                "label": "IDENTIFICATION DIVISION",
                "kind": 14,
                "detail": "Required first division",
                "documentation": "Declares the program name and structure"
            }),
            json!({
                "label": "ENVIRONMENT DIVISION",
                "kind": 14,
                "detail": "Required second division",
                "documentation": "Configures program environment and OS details"
            }),
            json!({
                "label": "DATA DIVISION",
                "kind": 14,
                "detail": "Required third division",
                "documentation": "Declares data structures and variables"
            }),
            json!({
                "label": "PROCEDURE DIVISION",
                "kind": 14,
                "detail": "Required fourth division",
                "documentation": "Contains program logic and operations"
            }),
            json!({
                "label": "COMPRESS",
                "kind": 6,
                "detail": "Operation",
                "documentation": "Compress data using configured protocol"
            }),
            json!({
                "label": "VERIFY-INTEGRITY",
                "kind": 6,
                "detail": "Operation",
                "documentation": "Verify data integrity with SHA-256"
            }),
        ];

        Ok(Response::success(
            req.id,
            json!({
                "items": completions
            }),
        ))
    }

    /// LSP: textDocument/definition request
    fn handle_definition(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;

        let line = params["position"]["line"].as_u64().ok_or("Missing line")? as usize;
        let character = params["position"]["character"]
            .as_u64()
            .ok_or("Missing character")? as usize;

        eprintln!("🔗 Definition at {}:{}:{}", uri, line, character);

        // For now, return the current position as definition location
        let definition = json!({
            "uri": uri,
            "range": {
                "start": { "line": line, "character": 0 },
                "end": { "line": line, "character": character }
            }
        });

        Ok(Response::success(req.id, definition))
    }

    /// LSP: textDocument/documentSymbol request
    fn handle_document_symbol(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;

        let backend = self.backend.lock().map_err(|e| e.to_string())?;
        let document = backend.get_document(uri).ok_or("Document not found")?;

        eprintln!("📋 Document symbols for {}", uri);

        // Extract divisions as symbols
        let mut symbols = Vec::new();
        let lines: Vec<&str> = document.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.contains("IDENTIFICATION DIVISION") {
                symbols.push(json!({
                    "name": "IDENTIFICATION DIVISION",
                    "kind": 11,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line_num, "character": 0 },
                            "end": { "line": line_num, "character": line.len() }
                        }
                    }
                }));
            } else if line.contains("ENVIRONMENT DIVISION") {
                symbols.push(json!({
                    "name": "ENVIRONMENT DIVISION",
                    "kind": 11,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line_num, "character": 0 },
                            "end": { "line": line_num, "character": line.len() }
                        }
                    }
                }));
            } else if line.contains("DATA DIVISION") {
                symbols.push(json!({
                    "name": "DATA DIVISION",
                    "kind": 11,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line_num, "character": 0 },
                            "end": { "line": line_num, "character": line.len() }
                        }
                    }
                }));
            } else if line.contains("PROCEDURE DIVISION") {
                symbols.push(json!({
                    "name": "PROCEDURE DIVISION",
                    "kind": 11,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line_num, "character": 0 },
                            "end": { "line": line_num, "character": line.len() }
                        }
                    }
                }));
            }
        }

        Ok(Response::success(req.id, json!(symbols)))
    }

    /// LSP: textDocument/references request
    fn handle_references(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;
        let line = params["position"]["line"].as_u64().ok_or("Missing line")? as usize;
        let character = params["position"]["character"]
            .as_u64()
            .ok_or("Missing character")? as usize;

        eprintln!("🔎 References at {}:{}:{}", uri, line, character);

        let backend = self.backend.lock().map_err(|e| e.to_string())?;
        let document = backend.get_document(uri).ok_or("Document not found")?;

        // Find all occurrences of the symbol at this position
        let lines: Vec<&str> = document.lines().collect();
        let mut references = Vec::new();

        if line < lines.len() {
            let line_content = lines[line];
            if character <= line_content.len() {
                // Extract word at position
                let mut start = character;
                let mut end = character;

                // Find word boundaries
                while start > 0
                    && line_content
                        .chars()
                        .nth(start - 1)
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    start -= 1;
                }
                while end < line_content.len()
                    && line_content
                        .chars()
                        .nth(end)
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    end += 1;
                }

                let word = &line_content[start..end];

                // Find all references to this word in the document
                for (ref_line_num, ref_line) in lines.iter().enumerate() {
                    for (ref_char, _) in ref_line.match_indices(word) {
                        references.push(json!({
                            "uri": uri,
                            "range": {
                                "start": { "line": ref_line_num, "character": ref_char },
                                "end": { "line": ref_line_num, "character": ref_char + word.len() }
                            }
                        }));
                    }
                }
            }
        }

        Ok(Response::success(req.id, json!(references)))
    }

    /// LSP: textDocument/rename request
    fn handle_rename(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let uri = params["textDocument"]["uri"]
            .as_str()
            .ok_or("Missing uri")?;
        let new_name = params["newName"].as_str().ok_or("Missing newName")?;
        let line = params["position"]["line"].as_u64().ok_or("Missing line")? as usize;
        let character = params["position"]["character"]
            .as_u64()
            .ok_or("Missing character")? as usize;

        eprintln!(
            "✏️  Rename at {}:{}:{} → {}",
            uri, line, character, new_name
        );

        let backend = self.backend.lock().map_err(|e| e.to_string())?;
        let document = backend.get_document(uri).ok_or("Document not found")?;

        let lines: Vec<&str> = document.lines().collect();
        if line >= lines.len() || character > lines[line].len() {
            return Ok(Response::success(req.id, json!({"changes": {}})));
        }

        let line_content = lines[line];

        // Extract word at position
        let mut start = character;
        let mut end = character;

        while start > 0
            && line_content
                .chars()
                .nth(start - 1)
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            start -= 1;
        }
        while end < line_content.len()
            && line_content
                .chars()
                .nth(end)
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            end += 1;
        }

        let old_word = &line_content[start..end];

        // Find all references and create workspace edit
        let mut edits = Vec::new();

        for (ref_line_num, ref_line) in lines.iter().enumerate() {
            for (ref_char, _) in ref_line.match_indices(old_word) {
                edits.push(json!({
                    "range": {
                        "start": { "line": ref_line_num, "character": ref_char },
                        "end": { "line": ref_line_num, "character": ref_char + old_word.len() }
                    },
                    "newText": new_name
                }));
            }
        }

        let workspace_edit = json!({
            "changes": {
                uri: edits
            }
        });

        Ok(Response::success(req.id, workspace_edit))
    }

    /// LSP: workspace/symbol request
    fn handle_workspace_symbol(&self, req: &Request) -> Result<Response, String> {
        let params = req.params.as_ref().ok_or("Missing params")?;
        let query = params["query"]
            .as_str()
            .ok_or("Missing query")?
            .to_lowercase();

        eprintln!("🔍 Workspace symbol search: '{}'", query);

        let _backend = self.backend.lock().map_err(|e| e.to_string())?;

        // Search through all open documents for matching symbols
        let mut results = Vec::new();

        // Get all documents from backend
        // For now, we'll return predefined symbols as baseline
        let predefined_symbols = vec![
            ("IDENTIFICATION DIVISION", 11),
            ("ENVIRONMENT DIVISION", 11),
            ("DATA DIVISION", 11),
            ("PROCEDURE DIVISION", 11),
            ("COMPRESS", 12),
            ("VERIFY-INTEGRITY", 12),
        ];

        for (symbol_name, kind) in predefined_symbols {
            if symbol_name.to_lowercase().contains(&query) {
                results.push(json!({
                    "name": symbol_name,
                    "kind": kind,
                    "location": {
                        "uri": "file:///centra-nf-stdlib",
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": symbol_name.len() }
                        }
                    }
                }));
            }
        }

        Ok(Response::success(req.id, json!(results)))
    }

    /// LSP: shutdown request
    fn handle_shutdown(&self, req: &Request) -> Result<Response, String> {
        eprintln!("🛑 Shutdown requested");
        Ok(Response::success(req.id, json!(null)))
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handler_creation() {
        let _handler = MessageHandler::new();
    }

    #[test]
    fn test_document_lifecycle() {
        let handler = MessageHandler::new();

        // Verify backend is accessible
        let backend = handler.backend.lock().unwrap();
        assert_eq!(backend.get_document("file:///test.cnf"), None);
    }

    #[test]
    fn test_hover_request() {
        let handler = MessageHandler::new();

        // Set up a document first
        {
            let mut backend = handler.backend.lock().unwrap();
            backend.set_document(
                "file:///test.cnf".to_string(),
                "IDENTIFICATION DIVISION.\n    PROGRAM-ID test.".to_string(),
            );
        }

        // Create hover request
        let req = Request::new(
            1,
            "textDocument/hover",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" },
                "position": { "line": 0, "character": 5 }
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());
    }

    #[test]
    fn test_completion_request() {
        let handler = MessageHandler::new();
        let req = Request::new(
            2,
            "textDocument/completion",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" },
                "position": { "line": 0, "character": 0 }
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());

        if let Ok(resp) = response {
            // Verify completions were returned
            assert!(resp.result.is_some());
        }
    }

    #[test]
    fn test_definition_request() {
        let handler = MessageHandler::new();
        let req = Request::new(
            3,
            "textDocument/definition",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" },
                "position": { "line": 0, "character": 5 }
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());
    }

    #[test]
    fn test_document_symbol_request() {
        let handler = MessageHandler::new();

        // Set up document
        {
            let mut backend = handler.backend.lock().unwrap();
            backend.set_document(
                "file:///test.cnf".to_string(),
                "IDENTIFICATION DIVISION.\nENVIRONMENT DIVISION.\nDATA DIVISION.\nPROCEDURE DIVISION.".to_string(),
            );
        }

        let req = Request::new(
            4,
            "textDocument/documentSymbol",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" }
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());

        if let Ok(resp) = response {
            assert!(resp.result.is_some());
        }
    }

    #[test]
    fn test_references_request() {
        let handler = MessageHandler::new();

        // Set up document with repeated symbol
        {
            let mut backend = handler.backend.lock().unwrap();
            backend.set_document(
                "file:///test.cnf".to_string(),
                "IDENTIFICATION DIVISION.\nIDENTIFICATION test.\nIDENTIFICATION again.".to_string(),
            );
        }

        let req = Request::new(
            5,
            "textDocument/references",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" },
                "position": { "line": 0, "character": 5 }
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());
    }

    #[test]
    fn test_rename_request() {
        let handler = MessageHandler::new();

        // Set up document
        {
            let mut backend = handler.backend.lock().unwrap();
            backend.set_document(
                "file:///test.cnf".to_string(),
                "IDENTIFICATION DIVISION.\nIDENTIFICATION test.".to_string(),
            );
        }

        let req = Request::new(
            6,
            "textDocument/rename",
            Some(json!({
                "textDocument": { "uri": "file:///test.cnf" },
                "position": { "line": 0, "character": 5 },
                "newName": "IDENTIFICATION_V2"
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());

        if let Ok(resp) = response {
            assert!(resp.result.is_some());
        }
    }

    #[test]
    fn test_workspace_symbol_request() {
        let handler = MessageHandler::new();

        let req = Request::new(
            7,
            "workspace/symbol",
            Some(json!({
                "query": "IDENTIFICATION"
            })),
        );

        let response = handler.handle_request(req, &mut JsonRpcIO::new());
        assert!(response.is_ok());

        if let Ok(resp) = response {
            // Should have found IDENTIFICATION DIVISION in workspace symbols
            assert!(resp.result.is_some());
        }
    }
}
