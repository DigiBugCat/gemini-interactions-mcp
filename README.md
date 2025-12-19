# Gemini Interactions MCP Server

A FastMCP server that provides web search and grounded AI answers using Google's Gemini API with Google Search grounding.

## Features

- **`ask`** - Get AI-synthesized answers grounded with Google Search using Gemini 3 Flash
- **`ask_thinking`** - Get answers with extended thinking for complex reasoning tasks

## Setup

### 1. Get a Gemini API Key

Get your API key from [Google AI Studio](https://aistudio.google.com/app/apikey).

### 2. Install Dependencies

```bash
# Using UV (recommended)
uv pip install -e .

# Or using pip
pip install -e .
```

### 3. Configure Environment

```bash
cp .env.example .env
# Edit .env and add your GEMINI_API_KEY
```

## Usage

### Run Locally

```bash
uv run fastmcp run server.py
```

### Interactive Development

```bash
fastmcp dev server.py
```

### Inspect Tools

```bash
fastmcp inspect server.py
```

## Claude Desktop Integration

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "gemini": {
      "command": "uv",
      "args": ["run", "fastmcp", "run", "/absolute/path/to/gemini-interactions-mcp/server.py"],
      "env": {
        "GEMINI_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

## Tools

### `ask`

Get AI-synthesized answers grounded with Google Search.

**Parameters:**
- `query` (required): Your question or prompt
- `max_tokens` (default: 4096): Maximum response length

**Returns:** AI-generated answer with source citations

### `ask_thinking`

Get answers with extended thinking for complex reasoning tasks.

**Parameters:**
- `query` (required): Your question or complex problem
- `max_tokens` (default: 8192): Maximum response length

**Returns:** AI-generated answer with reasoning process

**Note:** The thinking model does not support grounding, so answers are based on the model's training data only.

## Models Used

| Tool | Model | Grounding |
|------|-------|-----------|
| `ask` | gemini-3-flash-preview | Yes (Google Search) |
| `ask_thinking` | gemini-2.0-flash-thinking-exp-01-21 | No |

## License

MIT
