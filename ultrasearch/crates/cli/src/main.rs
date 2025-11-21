use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use console::style;
use ipc::{
    FieldKind, QueryExpr, RangeExpr, RangeOp, RangeValue, SearchMode, SearchRequest, TermExpr,
    TermModifier,
};
use ipc::framing;
use ipc::{StatusRequest, StatusResponse};
use uuid::Uuid;

/// Debug / scripting CLI for UltraSearch IPC.
#[derive(Parser, Debug)]
#[command(name = "ultrasearch-cli", version, about = "UltraSearch debug/diagnostic client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a search query (IPC transport to be wired later).
    Search {
        /// Query string (simple term; planner will expand later).
        query: String,
        /// Limit results.
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
        /// Offset for pagination.
        #[arg(short = 'o', long, default_value_t = 0)]
        offset: u32,
        /// Search mode (auto/name/content/hybrid).
        #[arg(short, long, value_enum, default_value_t = ModeArg::Auto)]
        mode: ModeArg,
        /// Optional timeout in milliseconds.
        #[arg(long)]
        timeout_ms: Option<u64>,
    },
    /// Request service status.
    Status {},
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ModeArg {
    Auto,
    Name,
    Content,
    Hybrid,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Search { query, limit, offset, mode, timeout_ms } => {
            let req = build_search_request(&query, limit, offset, timeout_ms, mode);
            print_request(&req)?;
            send_stub(req)?.map(|resp| {
                println!("{}", style("Response (stubbed):").yellow());
                println!("{resp:#?}");
            })?;
        }
        Commands::Status {} => {
            let req = build_status_request();
            println!("{}", style("Sending status request (stub framing):").cyan());
            let framed = framing::encode_frame(&bincode::serialize(&req)?)?;
            let (_payload, _rem) = framing::decode_frame(&framed)?;
            // Stub: fabricate an empty response
            let resp = StatusResponse {
                id: req.id,
                volumes: vec![],
                last_index_commit_ts: None,
                scheduler_state: "unknown".into(),
                metrics: None,
            };
            print_status_response(&resp)?;
        }
    }
    Ok(())
}

fn build_search_request(
    query: &str,
    limit: u32,
    offset: u32,
    timeout_ms: Option<u64>,
    mode: ModeArg,
) -> SearchRequest {
    let term = QueryExpr::Term(TermExpr {
        field: None,
        value: query.to_string(),
        modifier: TermModifier::Term,
    });

    SearchRequest {
        id: Uuid::new_v4(),
        query: term,
        limit,
        offset,
        mode: match mode {
            ModeArg::Auto => SearchMode::Auto,
            ModeArg::Name => SearchMode::NameOnly,
            ModeArg::Content => SearchMode::Content,
            ModeArg::Hybrid => SearchMode::Hybrid,
        },
        timeout: timeout_ms.map(std::time::Duration::from_millis),
    }
}

fn print_request(req: &SearchRequest) -> Result<()> {
    println!("{}", style("Sending request (stub transport):").cyan());
    println!("{req:#?}");
    Ok(())
}

fn build_status_request() -> StatusRequest {
    StatusRequest { id: Uuid::new_v4() }
}

fn print_status_response(resp: &StatusResponse) -> Result<()> {
    println!("{}", style("Status response (stubbed):").yellow());
    println!("Volumes: {}", resp.volumes.len());
    println!("Scheduler: {}", resp.scheduler_state);
    Ok(())
}

/// Placeholder transport: bincode roundtrip to prove serialization.
fn send_stub(req: SearchRequest) -> Result<SearchRequest> {
    let bytes = bincode::serialize(&req)?;
    // Frame/unframe for parity with pipe protocol.
    let framed = framing::encode_frame(&bytes)?;
    let (payload, _rem) = framing::decode_frame(&framed)?;
    let back: SearchRequest = bincode::deserialize(&payload)?;
    Ok(back)
}
