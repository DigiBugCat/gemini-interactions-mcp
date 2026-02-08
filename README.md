# Gemini Interactions MCP

A FastMCP server for stateful Gemini conversations with automatic Google Search grounding.

## Features

- **Stateful Conversations**: Maintain context across queries via `interaction_id`
- **Google Search Grounding**: Model automatically searches the web when needed
- **URL Context**: Parse and analyze linked web pages
- **Thinking Levels**: Control reasoning depth (minimal, medium, high)

## Setup

```bash
uv sync
cp .env.example .env
# Edit .env with your GEMINI_API_KEY
```

Get your API key from: https://aistudio.google.com/app/apikey

## Usage

```bash
uv run python server.py
```

## Tools

| Tool | Thinking Level | Description |
|------|----------------|-------------|
| `search` | minimal | Quick web search, structured results |
| `ask` | medium | Balanced grounded answers |
| `ask_thinking` | high | Deep reasoning with grounding |

All tools support `interaction_id` for stateful follow-up conversations.

## Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "gemini": {
      "command": "uv",
      "args": ["run", "python", "/path/to/gemini-interactions-mcp/server.py"],
      "env": {
        "GEMINI_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

## License

MIT
