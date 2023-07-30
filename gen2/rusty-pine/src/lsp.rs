//! Language server protocol
use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::jsonrpc::Result as RPCResult;
use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, Command, CompletionItem, CompletionOptions,
    CompletionParams, CompletionResponse, DocumentChanges, ExecuteCommandOptions,
    ExecuteCommandParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    OneOf, OptionalVersionedTextDocumentIdentifier, Position, Range, ServerCapabilities,
    TextDocumentEdit, TextEdit, Url, WorkDoneProgressOptions, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> RPCResult<InitializeResult> {
        debug!("{}::Backend::initialize()", module_path!());

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: Some(true),
                        },
                        resolve_provider: None,
                    },
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["pine.exec".to_owned()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        debug!("{}::Backend::initialized()", module_path!());

        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> RPCResult<()> {
        debug!("{}::Backend::shutdown()", module_path!());

        Ok(())
    }

    async fn code_action(&self, params: CodeActionParams) -> RPCResult<Option<CodeActionResponse>> {
        debug!("{}::Backend::code_action()", module_path!());

        Ok(Some(vec![CodeActionOrCommand::Command(Command {
            title: "Run pine".to_owned(),
            command: "pine.exec".to_owned(),
            arguments: Some(vec![
                Value::String(params.text_document.uri.to_string()),
                Value::Number(params.range.start.line.into()),
                Value::Number(params.range.start.character.into()),
            ]),
        })]))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> RPCResult<Option<Value>> {
        debug!("{}::Backend::execute_command()", module_path!());
        debug!("{:#?}", params);

        let line = params.arguments[1].as_u64().unwrap() as u32 + 1;

        let mut edits = HashMap::new();
        edits.insert(
            Url::parse(params.arguments[0].as_str().unwrap()).unwrap(),
            vec![TextEdit {
                range: Range::new(Position::new(line, 0), Position::new(line, 0)),
                new_text: "new text here\n".to_owned(),
            }],
        );

        debug!(
            "{:#?}",
            WorkspaceEdit {
                changes: Some(edits.clone()),
                ..Default::default()
            }
        );

        match self
            .client
            .apply_edit(WorkspaceEdit {
                changes: Some(edits),
                ..Default::default()
            })
            .await
        {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }
}
