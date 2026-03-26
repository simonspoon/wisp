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
        action: Box<NodeAction>,
    },
    /// Show the document tree
    Tree,
    /// Save the document to a file
    Save {
        /// File path to save to
        path: String,
    },
    /// Load a document from a file
    Load {
        /// File path to load from
        path: String,
    },
    /// Manage component templates
    Components {
        #[command(subcommand)]
        action: ComponentAction,
    },
    /// Undo the last operation
    Undo,
    /// Redo the last undone operation
    Redo,
    /// Watch for state change notifications (streams until Ctrl+C)
    Watch,
    /// Interactive session — keeps connection open for multiple commands
    Session,
    /// Capture a screenshot of the canvas (PNG)
    Screenshot {
        /// Output file path (default: wisp-screenshot.png)
        #[arg(short, long, default_value = "wisp-screenshot.png")]
        out: String,
    },
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
        /// Stroke color
        #[arg(long)]
        stroke: Option<String>,
        /// Stroke width in pixels
        #[arg(long)]
        stroke_width: Option<f64>,
        /// Text content
        #[arg(long)]
        text: Option<String>,
        /// Font size
        #[arg(long)]
        font_size: Option<f64>,
        /// Font family
        #[arg(long)]
        font_family: Option<String>,
        /// Font weight
        #[arg(long)]
        font_weight: Option<u16>,
        /// Text color
        #[arg(long)]
        color: Option<String>,
        /// Text alignment: left, center, right
        #[arg(long)]
        text_align: Option<String>,
        /// Corner radius
        #[arg(long)]
        radius: Option<f64>,
        /// Opacity (0.0-1.0)
        #[arg(long)]
        opacity: Option<f64>,
        /// Z-index for stacking order
        #[arg(long)]
        z_index: Option<i32>,
        /// Clip children that overflow bounds
        #[arg(long)]
        clip: bool,
        /// Enable text wrapping (auto-size height)
        #[arg(long)]
        text_wrap: bool,
        /// Layout mode: none, flex
        #[arg(long)]
        layout_mode: Option<String>,
        /// Flex direction: row, column
        #[arg(long)]
        direction: Option<String>,
        /// Flex align items: start, center, end, stretch
        #[arg(long)]
        align: Option<String>,
        /// Flex justify content: start, center, end, stretch, space_between
        #[arg(long)]
        justify: Option<String>,
        /// Gap between flex children
        #[arg(long)]
        gap: Option<f64>,
        /// Padding (all sides)
        #[arg(long)]
        padding: Option<f64>,
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
        /// Stroke color
        #[arg(long)]
        stroke: Option<String>,
        /// Stroke width in pixels
        #[arg(long)]
        stroke_width: Option<f64>,
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
        /// Font family
        #[arg(long)]
        font_family: Option<String>,
        /// Font weight
        #[arg(long)]
        font_weight: Option<u16>,
        /// Text color
        #[arg(long)]
        color: Option<String>,
        /// Text alignment: left, center, right
        #[arg(long)]
        text_align: Option<String>,
        /// Corner radius
        #[arg(long)]
        radius: Option<f64>,
        /// Opacity (0.0-1.0)
        #[arg(long)]
        opacity: Option<f64>,
        /// Z-index for stacking order
        #[arg(long)]
        z_index: Option<i32>,
        /// Clip children that overflow bounds
        #[arg(long)]
        clip: bool,
        /// Enable text wrapping (auto-size height)
        #[arg(long)]
        text_wrap: bool,
        /// Layout mode: none, flex
        #[arg(long)]
        layout_mode: Option<String>,
        /// Flex direction: row, column
        #[arg(long)]
        direction: Option<String>,
        /// Flex align items: start, center, end, stretch
        #[arg(long)]
        align: Option<String>,
        /// Flex justify content: start, center, end, stretch, space_between
        #[arg(long)]
        justify: Option<String>,
        /// Gap between flex children
        #[arg(long)]
        gap: Option<f64>,
        /// Padding (all sides)
        #[arg(long)]
        padding: Option<f64>,
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

#[derive(Subcommand)]
enum ComponentAction {
    /// List available component templates
    List,
    /// Instantiate a component template
    Use {
        /// Template name (e.g. stat-card, button, nav-item, chart-bar)
        name: String,
        /// Parent node ID (defaults to root)
        #[arg(short, long)]
        parent: Option<String>,
        /// X position
        #[arg(short, long)]
        x: Option<f64>,
        /// Y position
        #[arg(short, long)]
        y: Option<f64>,
        /// Label text (for components that have a label)
        #[arg(long)]
        label: Option<String>,
        /// Value text (for stat-card, etc.)
        #[arg(long)]
        value: Option<String>,
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

async fn connect(
    url: &str,
) -> Result<
    (
        futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ),
    Box<dyn std::error::Error>,
> {
    let (ws_stream, _) = connect_async(url)
        .await
        .map_err(|e| format!("Failed to connect to {url}: {e}. Is the Wisp app running?"))?;
    Ok(ws_stream.split())
}

/// Send an RPC request and wait for the matching response.
async fn rpc_call(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    method: &str,
    params: Value,
    req_id: u64,
) -> Result<RpcResponse, Box<dyn std::error::Error>> {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: serde_json::json!(req_id),
    };

    let msg = serde_json::to_string(&req)?;
    write.send(Message::Text(msg.into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                if resp.id == serde_json::json!(req_id) {
                    return Ok(resp);
                }
            }
            // Skip notifications
        }
    }

    Err("Connection closed before response".into())
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Watch => run_watch(&cli).await,
        Commands::Session => run_session(&cli).await,
        _ => run_oneshot(&cli).await,
    }
}

async fn run_oneshot(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = connect(&cli.url).await?;
    let (method, params) = build_request(&cli.command)?;

    let resp = rpc_call(&mut write, &mut read, &method, params, 1).await?;
    format_response(resp, &cli.command, cli.json)?;

    write.close().await.ok();
    Ok(())
}

async fn run_watch(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(&cli.url).await.map_err(|e| {
        format!(
            "Failed to connect to {}: {e}. Is the Wisp app running?",
            cli.url
        )
    })?;
    let (_write, mut read) = ws_stream.split();

    eprintln!("Watching for state changes... (Ctrl+C to stop)");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            // Try to parse as notification
            if let Ok(notif) = serde_json::from_str::<RpcNotification>(&text) {
                if notif.method == "state.changed" {
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&notif.params)?);
                    } else {
                        format_notification(&notif.params);
                    }
                }
            }
            // Also print responses (from other clients) if in JSON mode
            else if cli.json {
                if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
            }
        }
    }

    Ok(())
}

fn format_notification(params: &Value) {
    let kind = params
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("unknown");
    match kind {
        "node.created" => {
            let id = params.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            println!("[created] {id}");
        }
        "node.edited" => {
            let id = params.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            println!("[edited] {id}");
        }
        "node.deleted" => {
            let id = params.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            println!("[deleted] {id}");
        }
        "node.moved" => {
            let id = params.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            let pid = params
                .get("new_parent_id")
                .and_then(|p| p.as_str())
                .unwrap_or("?");
            println!("[moved] {id} -> {pid}");
        }
        _ => {
            println!("[{kind}] {params}");
        }
    }
}

async fn run_session(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = connect(&cli.url).await?;

    eprintln!("Wisp session started. Type commands (e.g. 'tree', 'node add Header -t frame').");
    eprintln!("Type 'quit' or 'exit' to end session.");

    let mut line_buf = String::new();
    let mut req_id: u64 = 0;

    loop {
        eprint!("wisp> ");
        // Flush stderr prompt
        use std::io::Write;
        std::io::stderr().flush().ok();

        line_buf.clear();
        let n = tokio::task::spawn_blocking({
            let mut buf = line_buf.clone();
            move || {
                let n = std::io::stdin().read_line(&mut buf).unwrap_or(0);
                (n, buf)
            }
        })
        .await?;
        line_buf = n.1;
        if n.0 == 0 {
            break; // EOF
        }

        let trimmed = line_buf.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }

        // Parse the line as wisp CLI args (prepend "wisp" to make clap happy)
        let mut args = vec!["wisp".to_string()];
        if cli.json {
            args.push("--json".to_string());
        }
        // Simple shell-like splitting (respects quotes)
        args.extend(shell_split(trimmed));

        let session_cli = match Cli::try_parse_from(&args) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        // Don't allow nested session/watch
        match &session_cli.command {
            Commands::Session | Commands::Watch => {
                eprintln!("Cannot nest session/watch commands");
                continue;
            }
            _ => {}
        }

        let (method, params) = match build_request(&session_cli.command) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {e}");
                continue;
            }
        };

        req_id += 1;
        match rpc_call(&mut write, &mut read, &method, params, req_id).await {
            Ok(resp) => {
                if let Err(e) = format_response(resp, &session_cli.command, session_cli.json) {
                    eprintln!("format error: {e}");
                }
            }
            Err(e) => {
                eprintln!("rpc error: {e}");
                break;
            }
        }
    }

    write.close().await.ok();
    eprintln!("Session ended.");
    Ok(())
}

/// Basic shell-like argument splitting that respects double quotes.
fn shell_split(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars = s.chars().peekable();

    for c in chars {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn build_request(command: &Commands) -> Result<(String, Value), Box<dyn std::error::Error>> {
    match command {
        Commands::Tree => Ok(("tree.get".to_string(), serde_json::json!({}))),
        Commands::Save { path } => {
            let abs_path = std::path::Path::new(path)
                .canonicalize()
                .unwrap_or_else(|_| std::path::PathBuf::from(path));
            Ok((
                "doc.save".to_string(),
                serde_json::json!({"path": abs_path.to_string_lossy()}),
            ))
        }
        Commands::Load { path } => {
            let abs_path = std::path::Path::new(path)
                .canonicalize()
                .map_err(|e| format!("File not found: {path}: {e}"))?;
            Ok((
                "doc.load".to_string(),
                serde_json::json!({"path": abs_path.to_string_lossy()}),
            ))
        }
        Commands::Components { action } => match action {
            ComponentAction::List => Ok(("component.list".to_string(), serde_json::json!({}))),
            ComponentAction::Use {
                name,
                parent,
                x,
                y,
                label,
                value,
            } => {
                let mut params = serde_json::json!({
                    "name": name,
                    "parent_id": parent.as_deref().unwrap_or("00000000-0000-0000-0000-000000000000"),
                });
                if let Some(x) = x {
                    params["x"] = serde_json::json!(x);
                }
                if let Some(y) = y {
                    params["y"] = serde_json::json!(y);
                }
                if let Some(label) = label {
                    params["label"] = serde_json::json!(label);
                }
                if let Some(value) = value {
                    params["value"] = serde_json::json!(value);
                }
                Ok(("component.use".to_string(), params))
            }
        },
        Commands::Undo => Ok(("doc.undo".to_string(), serde_json::json!({}))),
        Commands::Redo => Ok(("doc.redo".to_string(), serde_json::json!({}))),
        Commands::Watch | Commands::Session => Err("watch/session are handled separately".into()),
        Commands::Screenshot { .. } => Ok(("doc.screenshot".to_string(), serde_json::json!({}))),
        Commands::Node { action } => match action.as_ref() {
            NodeAction::Add {
                name,
                node_type,
                parent,
                x,
                y,
                width,
                height,
                fill,
                stroke,
                stroke_width,
                text,
                font_size,
                font_family,
                font_weight,
                color,
                text_align,
                radius,
                opacity,
                z_index,
                clip,
                text_wrap,
                layout_mode,
                direction,
                align,
                justify,
                gap,
                padding,
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

                if fill.is_some()
                    || stroke.is_some()
                    || stroke_width.is_some()
                    || radius.is_some()
                    || opacity.is_some()
                    || z_index.is_some()
                    || *clip
                {
                    let mut style = serde_json::Map::new();
                    if let Some(fill) = fill {
                        style.insert("fill".into(), serde_json::json!(fill));
                    }
                    if let Some(s) = stroke {
                        style.insert("stroke".into(), serde_json::json!(s));
                    }
                    if let Some(sw) = stroke_width {
                        style.insert("stroke_width".into(), serde_json::json!(sw));
                    }
                    if let Some(r) = radius {
                        style.insert("corner_radius".into(), serde_json::json!(r));
                    }
                    if let Some(o) = opacity {
                        style.insert("opacity".into(), serde_json::json!(o));
                    }
                    if let Some(z) = z_index {
                        style.insert("z_index".into(), serde_json::json!(z));
                    }
                    if *clip {
                        style.insert("clip".into(), serde_json::json!(true));
                    }
                    params["style"] = Value::Object(style);
                }

                if text.is_some()
                    || font_size.is_some()
                    || font_family.is_some()
                    || font_weight.is_some()
                    || color.is_some()
                    || text_align.is_some()
                    || *text_wrap
                {
                    let mut typo = serde_json::Map::new();
                    if let Some(text) = text {
                        typo.insert("content".into(), serde_json::json!(text));
                    }
                    if let Some(fs) = font_size {
                        typo.insert("font_size".into(), serde_json::json!(fs));
                    }
                    if let Some(ff) = font_family {
                        typo.insert("font_family".into(), serde_json::json!(ff));
                    }
                    if let Some(fw) = font_weight {
                        typo.insert("font_weight".into(), serde_json::json!(fw));
                    }
                    if let Some(c) = color {
                        typo.insert("color".into(), serde_json::json!(c));
                    }
                    if let Some(ta) = text_align {
                        typo.insert("text_align".into(), serde_json::json!(ta));
                    }
                    if *text_wrap {
                        typo.insert("text_auto_size".into(), serde_json::json!(true));
                    }
                    params["typography"] = Value::Object(typo);
                }

                if layout_mode.is_some()
                    || direction.is_some()
                    || align.is_some()
                    || justify.is_some()
                    || gap.is_some()
                    || padding.is_some()
                {
                    let mut al = serde_json::Map::new();
                    if let Some(mode) = layout_mode {
                        al.insert("mode".into(), serde_json::json!(mode));
                    }
                    if let Some(dir) = direction {
                        al.insert("direction".into(), serde_json::json!(dir));
                    }
                    if let Some(a) = align {
                        al.insert("align_items".into(), serde_json::json!(a));
                    }
                    if let Some(j) = justify {
                        al.insert("justify_content".into(), serde_json::json!(j));
                    }
                    if let Some(g) = gap {
                        al.insert("gap".into(), serde_json::json!(g));
                    }
                    if let Some(p) = padding {
                        al.insert("padding".into(), serde_json::json!(p));
                    }
                    params["auto_layout"] = Value::Object(al);
                }

                Ok(("node.create".to_string(), params))
            }
            NodeAction::Edit {
                id,
                name,
                fill,
                stroke,
                stroke_width,
                x,
                y,
                width,
                height,
                text,
                font_size,
                font_family,
                font_weight,
                color,
                text_align,
                radius,
                opacity,
                z_index,
                clip,
                text_wrap,
                layout_mode,
                direction,
                align,
                justify,
                gap,
                padding,
            } => {
                let mut params = serde_json::json!({"id": id});

                if let Some(name) = name {
                    params["name"] = serde_json::json!(name);
                }
                // Only send explicitly-set layout fields (partial edit)
                if x.is_some() || y.is_some() || width.is_some() || height.is_some() {
                    let mut layout = serde_json::Map::new();
                    if let Some(x) = x {
                        layout.insert("x".into(), serde_json::json!(x));
                    }
                    if let Some(y) = y {
                        layout.insert("y".into(), serde_json::json!(y));
                    }
                    if let Some(w) = width {
                        layout.insert("width".into(), serde_json::json!(w));
                    }
                    if let Some(h) = height {
                        layout.insert("height".into(), serde_json::json!(h));
                    }
                    params["layout"] = Value::Object(layout);
                }
                if fill.is_some()
                    || stroke.is_some()
                    || stroke_width.is_some()
                    || radius.is_some()
                    || opacity.is_some()
                    || z_index.is_some()
                    || *clip
                {
                    let mut style = serde_json::Map::new();
                    if let Some(fill) = fill {
                        style.insert("fill".into(), serde_json::json!(fill));
                    }
                    if let Some(s) = stroke {
                        style.insert("stroke".into(), serde_json::json!(s));
                    }
                    if let Some(sw) = stroke_width {
                        style.insert("stroke_width".into(), serde_json::json!(sw));
                    }
                    if let Some(r) = radius {
                        style.insert("corner_radius".into(), serde_json::json!(r));
                    }
                    if let Some(o) = opacity {
                        style.insert("opacity".into(), serde_json::json!(o));
                    }
                    if let Some(z) = z_index {
                        style.insert("z_index".into(), serde_json::json!(z));
                    }
                    if *clip {
                        style.insert("clip".into(), serde_json::json!(true));
                    }
                    params["style"] = Value::Object(style);
                }
                if text.is_some()
                    || font_size.is_some()
                    || font_family.is_some()
                    || font_weight.is_some()
                    || color.is_some()
                    || text_align.is_some()
                    || *text_wrap
                {
                    let mut typo = serde_json::Map::new();
                    if let Some(text) = text {
                        typo.insert("content".into(), serde_json::json!(text));
                    }
                    if let Some(fs) = font_size {
                        typo.insert("font_size".into(), serde_json::json!(fs));
                    }
                    if let Some(ff) = font_family {
                        typo.insert("font_family".into(), serde_json::json!(ff));
                    }
                    if let Some(fw) = font_weight {
                        typo.insert("font_weight".into(), serde_json::json!(fw));
                    }
                    if let Some(c) = color {
                        typo.insert("color".into(), serde_json::json!(c));
                    }
                    if let Some(ta) = text_align {
                        typo.insert("text_align".into(), serde_json::json!(ta));
                    }
                    if *text_wrap {
                        typo.insert("text_auto_size".into(), serde_json::json!(true));
                    }
                    params["typography"] = Value::Object(typo);
                }

                if layout_mode.is_some()
                    || direction.is_some()
                    || align.is_some()
                    || justify.is_some()
                    || gap.is_some()
                    || padding.is_some()
                {
                    let mut al = serde_json::Map::new();
                    if let Some(mode) = layout_mode {
                        al.insert("mode".into(), serde_json::json!(mode));
                    }
                    if let Some(dir) = direction {
                        al.insert("direction".into(), serde_json::json!(dir));
                    }
                    if let Some(a) = align {
                        al.insert("align_items".into(), serde_json::json!(a));
                    }
                    if let Some(j) = justify {
                        al.insert("justify_content".into(), serde_json::json!(j));
                    }
                    if let Some(g) = gap {
                        al.insert("gap".into(), serde_json::json!(g));
                    }
                    if let Some(p) = padding {
                        al.insert("padding".into(), serde_json::json!(p));
                    }
                    params["auto_layout"] = Value::Object(al);
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

fn format_response(
    resp: RpcResponse,
    command: &Commands,
    json_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(error) = resp.error {
        eprintln!("Server error [{}]: {}", error.code, error.message);
        return Ok(());
    }

    let result = resp.result.unwrap_or(Value::Null);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    match command {
        Commands::Tree => {
            if let Some(tree) = result.get("tree").and_then(|t| t.as_str()) {
                println!("{tree}");
            }
        }
        Commands::Save { .. } => {
            if let Some(path) = result.get("path").and_then(|p| p.as_str()) {
                let bytes = result.get("bytes").and_then(|b| b.as_u64()).unwrap_or(0);
                println!("Saved to {path} ({bytes} bytes)");
            }
        }
        Commands::Load { .. } => {
            if let Some(path) = result.get("path").and_then(|p| p.as_str()) {
                println!("Loaded from {path}");
            }
        }
        Commands::Components { action } => match action {
            ComponentAction::List => {
                if let Some(components) = result.get("components").and_then(|c| c.as_array()) {
                    for comp in components {
                        let name = comp.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                        let desc = comp
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        println!("  {name:16} {desc}");
                    }
                }
            }
            ComponentAction::Use { name, .. } => {
                if let Some(ids) = result.get("ids").and_then(|i| i.as_array()) {
                    println!("Created {name} ({} nodes)", ids.len());
                    if let Some(root_id) = ids.first().and_then(|i| i.as_str()) {
                        println!("  root: {root_id}");
                    }
                }
            }
        },
        Commands::Undo => {
            let undo_n = result
                .get("undo_available")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let redo_n = result
                .get("redo_available")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            println!("Undone (undo: {undo_n}, redo: {redo_n})");
        }
        Commands::Redo => {
            let undo_n = result
                .get("undo_available")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let redo_n = result
                .get("redo_available")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            println!("Redone (undo: {undo_n}, redo: {redo_n})");
        }
        Commands::Node { action } => match action.as_ref() {
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
        Commands::Screenshot { out } => {
            if let Some(b64) = result.get("png_base64").and_then(|v| v.as_str()) {
                if b64.is_empty() {
                    eprintln!("Screenshot capture failed (no canvas element?)");
                    std::process::exit(1);
                }
                use base64::Engine;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| format!("Base64 decode error: {e}"))?;
                std::fs::write(out, &bytes).map_err(|e| format!("Failed to write {out}: {e}"))?;
                println!("Screenshot saved to {out} ({} bytes)", bytes.len());
            }
        }
        _ => {}
    }

    Ok(())
}
