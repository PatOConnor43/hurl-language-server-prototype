use std::{collections::HashMap, error::Error, str::FromStr, sync::Mutex};

use futures::sink::SinkExt;
use futures::StreamExt;
use log::{debug, error, info, LevelFilter};
use phf::phf_map;
use ropey::Rope;
use serde::Serialize;
use std::io::Write;
use tokio::io::AsyncWrite;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

mod models;

// Custom codec to parse LSP Messages
struct JsonRPCMessageCodec;

impl tokio_util::codec::Decoder for JsonRPCMessageCodec {
    type Item = lsp_types::LSPAny;
    type Error = Box<dyn Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match memchr::memmem::find(src, b"\r\n\r\n") {
            None => Ok(None),
            Some(index) => {
                let header_prefix = b"Content-Length: ";
                match memchr::memmem::find(src, header_prefix) {
                    None => Ok(None),
                    Some(header_index) => {
                        match std::str::from_utf8(&src[header_index + header_prefix.len()..index]) {
                            Err(e) => Err(Box::new(e)),
                            Ok(length_as_str) => match length_as_str.parse() {
                                Err(e) => Err(Box::new(e)),
                                Ok(content_length) => {
                                    debug!("Content-Length: {}", content_length);
                                    let total_length = header_index
                                        + header_prefix.len()
                                        + length_as_str.len()
                                        + 4
                                        + content_length;
                                    if src.len() < (total_length) {
                                        return Ok(None);
                                    }
                                    src.advance(total_length - content_length);
                                    let val_ref = &src[0..content_length];
                                    let val: Result<lsp_types::LSPAny, _> =
                                        serde_json::from_slice(val_ref);
                                    src.advance(content_length);
                                    match val {
                                        Err(e) => Err(Box::new(e)),
                                        Ok(parsed) => Ok(Some(parsed)),
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

impl<T: Serialize> tokio_util::codec::Encoder<T> for JsonRPCMessageCodec {
    type Error = Box<dyn Error>;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = serde_json::to_string(&item)?;

        // Reserve just enough space to hold the `Content-Length: ` and `\r\n\r\n` constants,
        // the length of the message, and the message body.
        dst.reserve(msg.len() + number_of_digits(msg.len()) + 20);
        let mut writer = dst.writer();
        write!(writer, "Content-Length: {}\r\n\r\n{}", msg.len(), msg)?;
        writer.flush()?;

        Ok(())
    }
}

fn number_of_digits(mut n: usize) -> usize {
    let mut num_digits = 0;

    while n > 0 {
        n /= 10;
        num_digits += 1;
    }

    num_digits
}

#[derive(Debug, Default, Clone)]
struct HurlSectionPositions {
    pub request: Option<usize>,
    pub response: Option<usize>,
    pub query: Option<usize>,
    pub form: Option<usize>,
    pub multipart: Option<usize>,
    pub cookies: Option<usize>,
    pub capture: Option<usize>,
    pub asserts: Option<usize>,
    pub basic_auth: Option<usize>,
    pub options: Option<usize>,
}
impl FromStr for HurlSectionPositions {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let asserts = memchr::memmem::find(s.as_bytes(), b"\n[Asserts]\n");
        Ok(HurlSectionPositions {
            // Adding 1 to the index because I don't care about the first \n. I care where
            // [Asserts] starts
            asserts: asserts.map(|index| index + 1),
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files: Mutex<HashMap<String, Rope>> = Mutex::new(HashMap::new());
    let mut positions: Mutex<HashMap<String, HurlSectionPositions>> = Mutex::new(HashMap::new());

    simple_logging::log_to_file("pat.log", LevelFilter::Debug)?;
    info!("Starting!");
    let input = tokio::io::stdin();
    let mut output = tokio::io::stdout();
    let mut framed_reader = tokio_util::codec::FramedRead::new(input, JsonRPCMessageCodec {});
    let mut framed_writer = tokio_util::codec::FramedWrite::new(output, JsonRPCMessageCodec {});
    //let a = framed_reader.next().await;
    //error!("{:?}", a);
    //let a = framed_reader.next().await;
    //error!("{:?}", a);
    //let a = framed_reader.next().await;
    //error!("{:?}", a);
    while let Some(msg) = framed_reader.next().await {
        match msg {
            Err(e) => error!("{}", e),
            Ok(msg) => {
                if msg.get("method").is_none() {
                    continue;
                }

                if let Some(method) = msg.get("method").unwrap().as_str() {
                    info!("Recieved {}", method);
                    match method {
                        "initialize" => {
                            let result = lsp_types::InitializeResult {
                                server_info: Some(lsp_types::ServerInfo {
                                    name: "hurlsp".to_string(),
                                    version: Some("0.0.1".to_string()),
                                }),
                                capabilities: lsp_types::ServerCapabilities {
                                    hover_provider: Some(
                                        lsp_types::HoverProviderCapability::Options(
                                            lsp_types::HoverOptions {
                                                work_done_progress_options:
                                                    (lsp_types::WorkDoneProgressOptions {
                                                        work_done_progress: Some(false),
                                                    }),
                                            },
                                        ),
                                    ),
                                    completion_provider: Some(lsp_types::CompletionOptions {
                                        resolve_provider: Some(true),
                                        ..lsp_types::CompletionOptions::default()
                                    }),
                                    text_document_sync: Some(
                                        lsp_types::TextDocumentSyncCapability::Options(
                                            lsp_types::TextDocumentSyncOptions {
                                                open_close: Some(true),
                                                change: Some(lsp_types::TextDocumentSyncKind::FULL),
                                                ..lsp_types::TextDocumentSyncOptions::default()
                                            },
                                        ),
                                    ),
                                    ..lsp_types::ServerCapabilities::default()
                                },
                                ..Default::default()
                            };

                            let _ = write_result(msg, result, &mut framed_writer).await;
                        }
                        "initialized" => {}
                        "textDocument/didChange" => {
                            if let Some(params) = msg.get("params") {
                                let parsed: Result<lsp_types::DidChangeTextDocumentParams, _> =
                                    serde_json::from_value(params.clone());
                                match parsed {
                                    Err(_) => {
                                        error!("Failed to parse textDocument/didChange params")
                                    }
                                    Ok(parsed) => {
                                        let uri = parsed.text_document.uri.to_string();
                                        let content = parsed.content_changes.last();
                                        if content.is_none() {
                                            continue;
                                        }
                                        let text = &content.unwrap().text;
                                        let content = Rope::from_str(text.as_str());
                                        let content_positions =
                                            HurlSectionPositions::from_str(text.as_str()).unwrap();
                                        match files.lock() {
                                            Ok(mut f) => {
                                                f.insert(uri.to_owned(), content.to_owned());
                                            }
                                            Err(e) => error!("Failed to lock file mutex: {}", e),
                                        }
                                        match positions.lock() {
                                            Ok(mut p) => {
                                                p.insert(
                                                    uri.to_owned(),
                                                    content_positions.to_owned(),
                                                );
                                            }
                                            Err(e) => error!("Failed to lock file mutex: {}", e),
                                        }
                                        if content_positions.asserts.is_none() {
                                            // If there's no assert section, there's no diagnostics
                                            // for now.
                                            continue;
                                        }
                                        let mut diagnotics: Vec<lsp_types::Diagnostic> = vec![];
                                        let make_diagnostic =
                                            |range: lsp_types::Range| -> lsp_types::Diagnostic {
                                                lsp_types::Diagnostic {
                                                    range,
                                                    severity: Some(
                                                        lsp_types::DiagnosticSeverity::ERROR,
                                                    ),
                                                    source: Some("LSP".to_string()),
                                                    message: "Invalid assert".to_string(),
                                                    ..Default::default()
                                                }
                                            };

                                        let assert_block_line = &content
                                            .char_to_line(content_positions.asserts.unwrap());
                                        for (offset, line) in
                                            content.lines_at(*assert_block_line).skip(1).enumerate()
                                        {
                                            let mut first_token = "".to_string();
                                            let mut done = false;
                                            let mut chars = line.chars();
                                            while !done {
                                                let c = chars.next();
                                                if c.is_none() {
                                                    done = true;
                                                    continue;
                                                }
                                                let c = c.unwrap();
                                                if ['\n', ' ', '#'].contains(&c) {
                                                    done = true;
                                                    continue;
                                                }
                                                first_token = format!("{}{}", first_token, c);
                                            }
                                            if !["", "jsonpath"].contains(&first_token.as_str()) {
                                                let diagnostic_line =
                                                    *assert_block_line + 1 + offset;
                                                info!("{diagnostic_line}");
                                                let end_character = if first_token.len() == 0 {
                                                    0
                                                } else {
                                                    first_token.len() - 1
                                                };
                                                diagnotics.push(make_diagnostic(
                                                    lsp_types::Range {
                                                        start: lsp_types::Position {
                                                            line: diagnostic_line
                                                                .try_into()
                                                                .unwrap(),
                                                            character: 0,
                                                        },
                                                        end: lsp_types::Position {
                                                            line: diagnostic_line
                                                                .try_into()
                                                                .unwrap(),
                                                            character: end_character
                                                                .try_into()
                                                                .unwrap(),
                                                        },
                                                    },
                                                ));
                                            }
                                        }

                                        let diagnotics_result =
                                            lsp_types::PublishDiagnosticsParams {
                                                uri: url::Url::parse(&uri).unwrap(),
                                                diagnostics: diagnotics,
                                                version: Some(parsed.text_document.version),
                                            };
                                        let _ = write_notification(
                                            "textDocument/publishDiagnostics".to_string(),
                                            diagnotics_result,
                                            &mut framed_writer,
                                        )
                                        .await;
                                    }
                                };
                            };
                        }
                        "textDocument/didOpen" => {
                            if let Some(params) = msg.get("params") {
                                let parsed: Result<lsp_types::DidOpenTextDocumentParams, _> =
                                    serde_json::from_value(params.clone());
                                match parsed {
                                    Err(_) => {
                                        error!("Failed to parse textDocument/didOpen params")
                                    }
                                    Ok(parsed) => {
                                        let uri = parsed.text_document.uri.to_string();
                                        let text = parsed.text_document.text;
                                        let content_positions =
                                            HurlSectionPositions::from_str(text.as_str()).unwrap();
                                        let content = Rope::from_str(text.as_str());
                                        match files.lock() {
                                            Ok(mut f) => {
                                                f.insert(uri.to_owned(), content);
                                            }
                                            Err(e) => error!("Failed to lock file mutex: {}", e),
                                        }
                                        match positions.lock() {
                                            Ok(mut p) => {
                                                p.insert(uri, content_positions);
                                            }
                                            Err(e) => error!("Failed to lock file mutex: {}", e),
                                        }
                                    }
                                };
                            };
                        }
                        "textDocument/completion" => {
                            if let Some(params) = msg.get("params") {
                                let parsed: Result<lsp_types::CompletionParams, _> =
                                    serde_json::from_value(params.clone());
                                match parsed {
                                    Err(_) => {
                                        error!("Failed to parse textDocument/completion params")
                                    }
                                    Ok(parsed) => {
                                        let uri = parsed
                                            .text_document_position
                                            .text_document
                                            .uri
                                            .to_string();

                                        let position_guard = positions.lock().unwrap();
                                        let p = position_guard.get(&uri);
                                        if p.is_none() {
                                            let result =
                                                lsp_types::CompletionResponse::Array(vec![]);
                                            let _ =
                                                write_result(msg, result, &mut framed_writer).await;
                                            continue;
                                        }
                                        let p = p.unwrap();
                                        if p.asserts.is_none() {
                                            let result =
                                                lsp_types::CompletionResponse::Array(vec![]);
                                            let _ =
                                                write_result(msg, result, &mut framed_writer).await;
                                            continue;
                                        }

                                        let guard = files.lock().unwrap();
                                        let content = guard.get(&uri);
                                        match content {
                                            None => {
                                                let result =
                                                    lsp_types::CompletionResponse::Array(vec![]);
                                                let _ =
                                                    write_result(msg, result, &mut framed_writer)
                                                        .await;
                                                continue;
                                            }
                                            Some(content) => {
                                                let (text, _, _, _) = content.chunk_at_char(0);
                                                if ropey::str_utils::char_to_line_idx(
                                                    text,
                                                    p.asserts.unwrap(),
                                                ) < parsed
                                                    .text_document_position
                                                    .position
                                                    .line
                                                    .try_into()
                                                    .unwrap()
                                                {
                                                    let items = get_static_completions();
                                                    let result =
                                                        lsp_types::CompletionResponse::Array(items);
                                                    let _ = write_result(
                                                        msg,
                                                        result,
                                                        &mut framed_writer,
                                                    )
                                                    .await;
                                                    continue;
                                                } else {
                                                    let result =
                                                        lsp_types::CompletionResponse::Array(
                                                            vec![],
                                                        );
                                                    let _ = write_result(
                                                        msg,
                                                        result,
                                                        &mut framed_writer,
                                                    )
                                                    .await;
                                                    continue;
                                                }
                                            }
                                        };
                                    }
                                }
                            }
                        }
                        "completionItem/resolve" => {
                            if let Some(params) = msg.get("params") {
                                let parsed: Result<lsp_types::CompletionItem, _> =
                                    serde_json::from_value(params.clone());
                                match parsed {
                                    Err(_) => {
                                        error!("Failed to parse completionItem/resolve params")
                                    }
                                    Ok(parsed) => {
                                        let _ = write_result(msg, parsed, &mut framed_writer).await;
                                    }
                                }
                            }
                        }
                        "textDocument/hover" => {
                            if let Some(params) = msg.get("params") {
                                let parsed: Result<lsp_types::HoverParams, _> =
                                    serde_json::from_value(params.clone());
                                match parsed {
                                    Err(_) => {
                                        error!("Failed to parse textDocument/hover params")
                                    }
                                    Ok(parsed) => {
                                        let uri = parsed
                                            .text_document_position_params
                                            .text_document
                                            .uri
                                            .to_string();
                                        let character_position =
                                            parsed.text_document_position_params.position.character;
                                        let line_position =
                                            parsed.text_document_position_params.position.line;
                                        let files_guard = files.lock().unwrap();
                                        let content = files_guard.get(&uri).unwrap();
                                        let line = content.line(line_position.try_into().unwrap());
                                        let mut start_index: usize =
                                            character_position.try_into().unwrap();
                                        let mut end_index: usize =
                                            character_position.try_into().unwrap();
                                        info!("{}", character_position);

                                        // Get start index of what we're hovering
                                        while start_index >= 1 && line.char(start_index - 1) != ' '
                                        {
                                            start_index = start_index - 1;
                                        }

                                        // Get end index of what we're hovering
                                        while line.char(end_index) != ' '
                                            && line.char(end_index) != '\n'
                                        {
                                            end_index = end_index + 1;
                                        }

                                        let slice = line.slice(start_index..end_index);
                                        let word = slice.to_string();
                                        let value = match DOCUMENTATION_MAP.get(word.as_str()) {
                                            None => "".to_string(),
                                            Some(value) => value.to_string(),
                                        };
                                        let result = lsp_types::Hover {
                                            contents: lsp_types::HoverContents::Markup(
                                                lsp_types::MarkupContent {
                                                    kind: lsp_types::MarkupKind::Markdown,
                                                    value,
                                                },
                                            ),
                                            range: Some(lsp_types::Range {
                                                start: lsp_types::Position {
                                                    line: line_position,
                                                    character: start_index.try_into().unwrap(),
                                                },
                                                end: lsp_types::Position {
                                                    line: line_position,
                                                    character: end_index.try_into().unwrap(),
                                                },
                                            }),
                                        };
                                        let _ = write_result(msg, result, &mut framed_writer).await;
                                    }
                                }
                            }
                        }

                        _ => error!("Unimplemented method: {}", method),
                    }
                }
            }
        }
    }

    info!("Exiting");

    Ok(())
}

async fn write_result<T, W>(
    msg: lsp_types::LSPAny,
    result: T,
    writer: &mut tokio_util::codec::FramedWrite<W, JsonRPCMessageCodec>,
) -> Result<(), Box<dyn Error>>
where
    W: std::marker::Unpin,
    W: AsyncWrite,
    T: Serialize,
{
    let id = msg.get("id").unwrap();
    let id = match id.as_i64() {
        Some(i) => i,
        None => match id.as_str() {
            Some(i) => i.parse::<i64>().unwrap_or(0),
            None => 0,
        },
    };
    let response = models::ResponseMessage::new(id, result);
    writer.send(response).await
}

async fn write_notification<T, W>(
    method: String,
    params: T,
    writer: &mut tokio_util::codec::FramedWrite<W, JsonRPCMessageCodec>,
) -> Result<(), Box<dyn Error>>
where
    W: std::marker::Unpin,
    W: AsyncWrite,
    T: Serialize,
{
    let notification = models::Notification::new(method, params);
    writer.send(notification).await
}

fn get_static_completions() -> Vec<lsp_types::CompletionItem> {
    vec![
        lsp_types::CompletionItem {
            label: "jsonpath".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            documentation: Some(lsp_types::Documentation::MarkupContent(
                lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: include_str!("static_documentation/jsonpath.md").to_string(),
                },
            )),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "status".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "url".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "header".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "cookie".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "body".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "xpath".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "regex".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "variable".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "duration".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "sha256".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "md5".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        lsp_types::CompletionItem {
            label: "bytes".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
    ]
}

const DOCUMENTATION_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "jsonpath" => include_str!("static_documentation/jsonpath.md"),
    "status" => include_str!("static_documentation/status.md"),
    "url" => include_str!("static_documentation/url.md"),
    "header" => include_str!("static_documentation/header.md"),
    "cookie" => include_str!("static_documentation/cookie.md"),
    "body" => include_str!("static_documentation/body.md"),
    "xpath" => include_str!("static_documentation/xpath.md"),
    "regex" => include_str!("static_documentation/regex.md"),
    "variable" => include_str!("static_documentation/variable.md"),
    "duration" => include_str!("static_documentation/duration.md"),
    "sha256" => include_str!("static_documentation/sha256.md"),
    "md5" => include_str!("static_documentation/md5.md"),
    "bytes" => include_str!("static_documentation/bytes.md"),
};
