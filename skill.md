# Gemini Ask Skill

Use `gemini-ask` to search the web and get grounded answers from Gemini 3 Flash with Google Search integration.

## When to Use

- **Web search**: Current events, recent news, facts that need verification
- **Grounded answers**: Questions needing authoritative sources
- **Deep reasoning**: Complex problems requiring step-by-step analysis
- **Follow-up questions**: Continue a conversation with previous context

## Commands

### Quick Search (minimal thinking)
```bash
gemini-ask --search "query"
```
Returns structured search results (title, URL, snippet).

### Grounded Answer (medium thinking)
```bash
gemini-ask --ask "question"
```
Returns a concise, factual answer with citations.

### Deep Reasoning (high thinking)
```bash
gemini-ask --think "complex question"
```
Returns thorough analysis with step-by-step reasoning.

### Continue Conversation
```bash
gemini-ask --ask "follow-up question" -i <interaction_id>
```
Use the `interaction_id` from a previous response to maintain context.

## Options

| Option | Description |
|--------|-------------|
| `--search <query>` | Quick web search (minimal thinking) |
| `--ask <query>` | Grounded answer (medium thinking) |
| `--think <query>` | Deep reasoning (high thinking) |
| `-i, --interaction <id>` | Previous interaction ID for follow-up |
| `-o, --output <format>` | Output format: `text` (default) or `json` |

## Subcommands

For more control, use subcommands:

```bash
# Search with custom max results
gemini-ask search "query" --max-results 5

# Ask with interaction context
gemini-ask ask "question" -i <interaction_id>

# Think with interaction context
gemini-ask think "question" -i <interaction_id>

# Follow up with specific thinking level
gemini-ask follow-up "question" -i <interaction_id> --thinking-level high
```

## Examples

### Search for current information
```bash
gemini-ask --search "latest AI news December 2024"
```

### Ask a factual question
```bash
gemini-ask --ask "What is the current population of Tokyo?"
```

### Deep analysis
```bash
gemini-ask --think "Compare the economic policies of Japan and South Korea"
```

### Multi-turn conversation
```bash
# First question
gemini-ask --ask "What is quantum computing?"
# Returns: ... interaction_id: v1_abc123

# Follow-up (Gemini remembers context)
gemini-ask --ask "How is it different from classical computing?" -i v1_abc123
```

## Output Format

Responses include:
- Answer text with inline citations
- Sources list (numbered URLs)
- Metadata: interaction_id, status, token usage

## Environment

Requires `GEMINI_API_KEY` environment variable. Get your key from https://aistudio.google.com/app/apikey
