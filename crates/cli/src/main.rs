use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use wisp_protocol::*;

const DEFAULT_URL: &str = "ws://127.0.0.1:9847/ws";

#[derive(Parser)]
#[command(name = "wisp", version, about = "Wisp design tool CLI")]
struct Cli {
    /// WebSocket server URL
    #[arg(long, default_value = DEFAULT_URL, global = true)]
    url: String,

    /// Output raw JSON instead of formatted text
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage nodes
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },
    /// Show the document tree
    Tree,
}

#[derive(Subcommand)]
enum NodeAction {
    /// Create a new node
    Add {
        /// Node name
        name: String,
        /// Node type: frame, text, rectangle, ellipse, group
        #[arg(short = 't', long, default_value = "frame")]
        node_type: String,
        /// Parent node ID (defaults to root)
        #[arg(short, long)]
        parent: Option<String>,
        /// X position
        #[arg(short, long)]
        x: Option<f64>,
        /// Y position
        #[arg(short, long)]
        y: Option<f64>,
        /// Width
        #[arg(long)]
        width: Option<f64>,
        /// Height
        #[arg(long)]
        height: Option<f64>,
        /// Fill color
        #[arg(long)]
        fill: Option<String>,
        /// Text content
        #[arg(long)]
        text: Option<String>,
        /// Font size
        #[arg(long)]
        font_size: Option<f64>,
        /// Corner radius
        #[arg(long)]
        radius: Option<f64>,
        /// Opacity (0.0-1.0)
        #[arg(long)]
        opacity: Option<f64>,
    },
    /// Edit a node
    Edit {
        /// Node ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// Fill color
        #[arg(long)]
        fill: Option<String>,
        /// X position
        #[arg(short, long)]
        x: Option<f64>,
        /// Y position
        #[arg(short, long)]
        y: Option<f64>,
        /// Width
        #[arg(long)]
        width: Option<f64>,
        /// Height
        #[arg(long)]
        height: Option<f64>,
        /// Text content
        #[arg(long)]
        text: Option<String>,
        /// Font size
        #[arg(long)]
        font_size: Option<f64>,
        /// Corner radius
        #[arg(long)]
        radius: Option<f64>,
        /// Opacity (0.0-1.0)
        #[arg(long)]
        opacity: Option<f64>,
    },
    /// Delete a node
    Delete {
        /// Node ID
        id: String,
    },
    /// Show details of a node
    Show {
        /// Node ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(&cli.url).await.map_err(|e| {
        format!(
            "Failed to connect to {}: {e}. Is the Wisp app running?",
            cli.url
        )
    })?;

    let (mut write, mut read) = ws_stream.split();

    let (method, params) = build_request(&cli)?;

    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method,
        params,
        id: serde_json::json!(1),
    };

    let msg = serde_json::to_string(&req)?;
    write.send(Message::Text(msg.into())).await?;

    // Read messages until we get our response (id == 1)
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            // Try to parse as RPC response
            if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                if resp.id == serde_json::json!(1) {
                    handle_response(resp, &cli)?;
                    break;
                }
            }
            // Skip notifications
        }
    }

    // Close cleanly
    write.close().await.ok();
    Ok(())
}

fn build_request(cli: &Cli) -> Result<(String, Value), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Tree => Ok(("tree.get".to_string(), serde_json::json!({}))),
        Commands::Node { action } => match action {
            NodeAction::Add {
                name,
                node_type,
                parent,
                x,
                y,
                width,
                height,
                fill,
                text,
                font_size,
                radius,
                opacity,
            } => {
                let mut params = serde_json::json!({
                    "name": name,
                    "node_type": node_type,
                    "parent_id": parent.as_deref().unwrap_or("00000000-0000-0000-0000-000000000000"),
                });

                if x.is_some() || y.is_some() || width.is_some() || height.is_some() {
                    params["layout"] = serde_json::json!({
                        "x": x.unwrap_or(0.0),
                        "y": y.unwrap_or(0.0),
                        "width": width.unwrap_or(0.0),
                        "height": height.unwrap_or(0.0),
                    });
                }

                if fill.is_some() || radius.is_some() || opacity.is_some() {
                    let mut style = serde_json::Map::new();
                    if let Some(fill) = fill {
                        style.insert("fill".into(), serde_json::json!(fill));
                    }
                    if let Some(r) = radius {
                        style.insert("corner_radius".into(), serde_json::json!(r));
                    }
                    if let Some(o) = opacity {
                        style.insert("opacity".into(), serde_json::json!(o));
                    }
                    params["style"] = Value::Object(style);
                }

                if text.is_some() || font_size.is_some() {
                    let mut typo = serde_json::Map::new();
                    if let Some(text) = text {
                        typo.insert("content".into(), serde_json::json!(text));
                    }
                    if let Some(fs) = font_size {
                        typo.insert("font_size".into(), serde_json::json!(fs));
                    }
                    params["typography"] = Value::Object(typo);
                }

                Ok(("node.create".to_string(), params))
            }
            NodeAction::Edit {
                id,
                name,
                fill,
                x,
                y,
                width,
                height,
                text,
                font_size,
                radius,
                opacity,
            } => {
                let mut params = serde_json::json!({"id": id});

                if let Some(name) = name {
                    params["name"] = serde_json::json!(name);
                }
                if x.is_some() || y.is_some() || width.is_some() || height.is_some() {
                    params["layout"] = serde_json::json!({
                        "x": x.unwrap_or(0.0),
                        "y": y.unwrap_or(0.0),
                        "width": width.unwrap_or(0.0),
                        "height": height.unwrap_or(0.0),
                    });
                }
                if fill.is_some() || radius.is_some() || opacity.is_some() {
                    let mut style = serde_json::Map::new();
                    if let Some(fill) = fill {
                        style.insert("fill".into(), serde_json::json!(fill));
                    }
                    if let Some(r) = radius {
                        style.insert("corner_radius".into(), serde_json::json!(r));
                    }
                    if let Some(o) = opacity {
                        style.insert("opacity".into(), serde_json::json!(o));
                    }
                    params["style"] = Value::Object(style);
                }
                if text.is_some() || font_size.is_some() {
                    let mut typo = serde_json::Map::new();
                    if let Some(text) = text {
                        typo.insert("content".into(), serde_json::json!(text));
                    }
                    if let Some(fs) = font_size {
                        typo.insert("font_size".into(), serde_json::json!(fs));
                    }
                    params["typography"] = Value::Object(typo);
                }

                Ok(("node.edit".to_string(), params))
            }
            NodeAction::Delete { id } => {
                Ok(("node.delete".to_string(), serde_json::json!({"id": id})))
            }
            NodeAction::Show { id } => Ok(("node.show".to_string(), serde_json::json!({"id": id}))),
        },
    }
}

fn handle_response(resp: RpcResponse, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(error) = resp.error {
        eprintln!("Server error [{}]: {}", error.code, error.message);
        std::process::exit(1);
    }

    let result = resp.result.unwrap_or(Value::Null);

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    // Format based on the command
    match &cli.command {
        Commands::Tree => {
            if let Some(tree) = result.get("tree").and_then(|t| t.as_str()) {
                println!("{tree}");
            }
        }
        Commands::Node { action } => match action {
            NodeAction::Add { .. } => {
                if let Some(id) = result.get("id").and_then(|i| i.as_str()) {
                    println!("Created node {id}");
                }
            }
            NodeAction::Show { .. } => {
                if let Some(node) = result.get("node") {
                    println!("{}", serde_json::to_string_pretty(node)?);
                }
            }
            NodeAction::Edit { .. } => {
                println!("Node updated");
            }
            NodeAction::Delete { .. } => {
                println!("Node deleted");
            }
        },
    }

    Ok(())
}
