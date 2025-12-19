# Gemini Interactions MCP

Two independent tools for stateful Gemini interactions with Google Search grounding:

1. **CLI Client (Rust)** - Standalone `gemini-ask` binary that calls Gemini API directly
2. **MCP Server (Python)** - FastMCP backend for tool-based integrations (Claude Desktop, etc.)

## Features

- **Stateful Conversations**: Maintain context across multiple queries via `interaction_id`
- **Google Search Grounding**: Automatic web search for current/factual information
- **URL Context**: Parse and analyze linked web pages
- **Thinking Levels**: Control reasoning depth (minimal, low, medium, high)

## Quick Start

### CLI Usage

```bash
# Set API key
export GEMINI_API_KEY=your_key_here

# Quick search
gemini-ask --search "latest AI news"

# Get grounded answer
gemini-ask --ask "What is quantum computing?"

# Deep reasoning
gemini-ask --think "Compare quantum vs classical computing"

# Follow-up conversation
gemini-ask --ask "Can you explain more?" -i <interaction_id>
```

### MCP Server Usage

```bash
cd server
uv run python server.py
```

Then connect via MCP client with tools: `search`, `ask`, `ask_thinking`, `follow_up`, `upload_files`

## Architecture

The CLI and MCP Server are **independent** - use whichever fits your workflow:

```
┌─────────────────────────────────────────────────────────┐
│  CLI Client (Rust) - gemini-ask                         │
│  - Standalone binary, no dependencies                   │
│  - Direct Gemini API calls via HTTP                     │
│  - Use as Claude Code skill or command line             │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│  Gemini Interactions API                                │
│  - Model: gemini-3-flash-preview                        │
│  - Automatic grounding (google_search, url_context)     │
│  - Server-side conversation state                       │
└─────────────────────┬───────────────────────────────────┘
                      ▲
                      │
┌─────────────────────┴───────────────────────────────────┐
│  MCP Server (Python/FastMCP) - optional                 │
│  - For Claude Desktop, Cursor, etc.                     │
│  - Same API features via MCP tools                      │
└─────────────────────────────────────────────────────────┘
```

## Installation

### CLI (Rust)

```bash
# Install globally
cd cli
cargo install --path .

# Or build locally
cargo build --release
# Binary at: ./target/release/gemini-ask
```

### MCP Server (Python)

```bash
cd server
uv sync
cp .env.example .env
# Edit .env with your GEMINI_API_KEY
```

## API Key

Get your Gemini API key from: https://aistudio.google.com/app/apikey

## Tools

| Tool | Thinking Level | Description |
|------|----------------|-------------|
| `search` | minimal | Quick search, structured results (~3-7s) |
| `ask` | medium | Balanced grounded answers (~8-12s) |
| `ask_thinking` | high | Deep reasoning with grounding (~10-15s) |
| `follow_up` | configurable | Continue previous conversation |
| `upload_files` | N/A | Upload files for analysis |

## Thinking Levels

- **minimal**: Fast, low latency (~3-7s with grounding)
- **low**: Light reasoning
- **medium**: Balanced (default, ~10-15s with grounding)
- **high**: Deep reasoning for complex problems (~10-15s)

## Stateful Conversations

The Interactions API maintains server-side state:

```bash
# First query
gemini-ask --ask "What is Stoicism?"
# Returns: interaction_id: v1_abc123

# Follow-up (Gemini remembers context)
gemini-ask --ask "Who were its main practitioners?" -i v1_abc123
```

Benefits:
- No need to resend conversation history
- Implicit caching (faster, cheaper)
- 55-day retention (paid) / 1-day (free)

## Claude Desktop Integration

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "gemini": {
      "command": "uv",
      "args": ["run", "python", "/path/to/gemini-interactions-mcp/server/server.py"],
      "env": {
        "GEMINI_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

## Project Structure

```
gemini-interactions-mcp/
├── server/
│   ├── server.py          # MCP server (Python/FastMCP)
│   ├── test_server.py     # Server tests
│   ├── pyproject.toml
│   └── .env.example
├── cli/
│   ├── src/
│   │   └── main.rs        # Rust CLI
│   └── Cargo.toml
├── skill.md               # Claude Code skill
└── README.md
```

## License

MIT
