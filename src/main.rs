use std::{collections::HashMap, panic};
use std::fs::{self, File};
use std::io;

use log::{debug, error, info, trace, warn};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::*;
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    Diagnostic, PublishDiagnosticsParams, Url,
};
use lsp_types::{
    notification::{Notification as _, *},
    request::{Request as RequestTrait, *},
    *,
};
use parser::AST;
use simplelog::WriteLogger;

mod lexer;
mod parser;

use parser::ParseError;

type DynResult<T, E = Box<dyn std::error::Error>> = Result<T, E>;

fn main() {
    run().unwrap()
}

fn run() -> DynResult<()> {
    let data_dir = dirs_next::data_dir()
        .expect("Failed to find data_dir")
        .join("test_lsp_server");
    let file_path = data_dir.join("lsp_server.log");

    if !data_dir.exists() {
        fs::create_dir_all(data_dir).expect("Failed to create data dir");
    }

    WriteLogger::init(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        File::create(file_path).expect("Failed to create log file"),
    )
    .expect("Failed to start logger");

    panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));

    let (connection, io_threads) = Connection::stdio();
    let capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
        ..ServerCapabilities::default()
    })
    .unwrap();

    connection.initialize(capabilities)?;

    Server {
        files: HashMap::new(),
        conn: connection,
    }
    .run();

    io_threads.join()?;

    Ok(())
}

struct Server {
    files: HashMap<Url, (AST, String)>,
    conn: Connection,
}

impl Server {
    fn run(&mut self) {
        while let Ok(msg) = self.conn.receiver.recv() {
            trace!("Message: {:#?}", msg);
            match msg {
                Message::Request(req) => {
                    let id = req.id.clone();
                    match self.conn.handle_shutdown(&req) {
                        Ok(true) => break,
                        Ok(false) => {
                            if let Err(err) = self.handle_request(req) {
                                self.err(id, err);
                            }
                        }
                        Err(err) => {
                            // This only fails if a shutdown was
                            // requested in the first place, so it
                            // should definitely break out of the
                            // loop.
                            self.err(id, err);
                            break;
                        }
                    }
                }
                Message::Notification(notification) => {
                    let _ = self.handle_notification(notification);
                }
                Message::Response(_) => (),
            }
        }
    }

    fn handle_notification(&mut self, req: Notification) -> DynResult<()> {
        debug!("method: {:#?}", &*req.method);
        match &*req.method {
            DidOpenTextDocument::METHOD => {
                info!("did open");
                let params: DidOpenTextDocumentParams = serde_json::from_value(req.params)?;
                let text = params.text_document.text;
                // let parsed = parser::parse(&text);
                // trace!("Parsed: {:#?}", parsed);
                // self.send_diagnostics(params.text_document.uri.clone(), &text, &parsed)?;
                self.send_simple(params.text_document.uri.clone(), "did open");
                // self.files.insert(params.text_document.uri, (parsed, text));
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(req.params)?;
                if let Some(change) = params.content_changes.into_iter().last() {
                    // let parsed = parser::parse(&change.text);
                    // debug!("Parsed: {:#?}", parsed);
                    self.send_simple(params.text_document.uri.clone(), &change.text);
                    // self.send_diagnostics(params.text_document.uri.clone(), &change.text, &parsed)?;
                    // self.files
                    //     .insert(params.text_document.uri, (parsed, change.text));
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn send_simple(&mut self, uri: Url, text: &str) -> DynResult<()> {
        info!("sending diagnostics");
        let mut diagnostics = Vec::new();
        diagnostics.push(Diagnostic {
            range: Range::new(Position::new(2, 2), Position::new(2, 2)),
            severity: Some(DiagnosticSeverity::Error),
            message: text.to_string(),
            ..Diagnostic::default()
        });
        self.notify(Notification::new(
            "textDocument/publishDiagnostics".into(),
            PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: None,
            },
        ));
        Ok(())
    }

    fn send_diagnostics(&mut self, uri: Url, code: &str, ast: &AST) -> DynResult<()> {
        info!("sending diagnostics");
        let errors = ast.errors();
        let mut diagnostics = Vec::with_capacity(errors.len());
        for err in errors {
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(2, 2), Position::new(2, 2)),
                severity: Some(DiagnosticSeverity::Error),
                message: err.to_string(),
                ..Diagnostic::default()
            });
        }
        self.notify(Notification::new(
            "textDocument/publishDiagnostics".into(),
            PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: None,
            },
        ));
        Ok(())
    }

    fn handle_request(&mut self, req: Request) -> DynResult<()> {
        trace!("Handling request {:#?}", req);
        Ok(())
    }

    fn reply(&mut self, response: Response) {
        trace!("Sending response: {:#?}", response);
        self.conn.sender.send(Message::Response(response)).unwrap();
    }

    fn notify(&mut self, notification: Notification) {
        trace!("Sending notification: {:#?}", notification);
        self.conn
            .sender
            .send(Message::Notification(notification))
            .unwrap();
    }

    fn err<E>(&mut self, id: RequestId, err: E)
    where
        E: std::fmt::Display,
    {
        warn!("{}", err);
        self.reply(Response::new_err(
            id,
            ErrorCode::UnknownErrorCode as i32,
            err.to_string(),
        ));
    }
}
