"""
Gemini Interactions MCP Server

A FastMCP server that provides web search and grounded AI answers using Google's Gemini API
with Google Search grounding.

Tools:
1. ask - Get AI-synthesized answers grounded with Google Search
2. ask_thinking - Get answers with extended thinking for complex reasoning
"""

import os
from typing import Optional
from fastmcp import FastMCP
import httpx
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Initialize FastMCP server
mcp = FastMCP("Gemini Research")

# Get API key from environment
GEMINI_API_KEY = os.getenv("GEMINI_API_KEY")
if not GEMINI_API_KEY:
    raise ValueError(
        "GEMINI_API_KEY environment variable is required. "
        "Get your API key from https://aistudio.google.com/app/apikey"
    )

# API configuration
GEMINI_API_BASE = "https://generativelanguage.googleapis.com/v1beta"


def get_endpoint(model: str) -> str:
    """Get the API endpoint for a given model."""
    return f"{GEMINI_API_BASE}/models/{model}:generateContent"


def format_response(response: dict) -> str:
    """Format Gemini response with grounding metadata."""
    try:
        candidate = response.get("candidates", [{}])[0]
        content = candidate.get("content", {})
        parts = content.get("parts", [])

        # Extract text from parts
        text_parts = []
        for part in parts:
            if "text" in part:
                text_parts.append(part["text"])

        answer = "\n".join(text_parts) if text_parts else "No response generated."
        output = [answer]

        # Extract grounding metadata
        grounding = candidate.get("groundingMetadata", {})

        # Add search queries used
        if queries := grounding.get("webSearchQueries"):
            output.append("\n\n🔍 Search Queries:")
            for query in queries:
                output.append(f"- {query}")

        # Add sources from grounding chunks
        if chunks := grounding.get("groundingChunks"):
            output.append("\n\n📚 Sources:")
            seen_urls = set()
            for i, chunk in enumerate(chunks, 1):
                if web := chunk.get("web"):
                    uri = web.get("uri", "")
                    title = web.get("title", "Untitled")
                    if uri and uri not in seen_urls:
                        seen_urls.add(uri)
                        output.append(f"{i}. [{title}]({uri})")

        return "\n".join(output)

    except Exception as e:
        return f"Error parsing response: {str(e)}\n\nRaw response: {response}"


def _generate_content(
    query: str,
    model: str,
    max_tokens: int = 4096,
    system_prompt: Optional[str] = None,
    use_grounding: bool = True,
) -> str:
    """Helper function for Gemini API calls."""
    try:
        headers = {
            "x-goog-api-key": GEMINI_API_KEY,
            "Content-Type": "application/json"
        }

        # Build contents array
        contents = []
        if system_prompt:
            contents.append({
                "role": "user",
                "parts": [{"text": system_prompt}]
            })
            contents.append({
                "role": "model",
                "parts": [{"text": "Understood. I will follow these instructions."}]
            })

        contents.append({
            "role": "user",
            "parts": [{"text": query}]
        })

        payload = {
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": max_tokens,
            }
        }

        # Add Google Search grounding tool
        if use_grounding:
            payload["tools"] = [{"google_search": {}}]

        endpoint = get_endpoint(model)

        with httpx.Client(timeout=120.0) as client:
            response = client.post(endpoint, json=payload, headers=headers)
            response.raise_for_status()
            data = response.json()

        return format_response(data)

    except httpx.HTTPStatusError as e:
        return f"API error: {e.response.status_code} - {e.response.text}"
    except Exception as e:
        return f"Request failed: {str(e)}"


@mcp.tool
def ask(
    query: str,
    max_tokens: int = 8192,
) -> str:
    """
    Get AI-synthesized answers grounded with Google Search.

    Uses Gemini 3 Flash with automatic Google Search grounding for accurate,
    up-to-date information with citations.

    Args:
        query: Your question or prompt
        max_tokens: Maximum response length (default: 8192)

    Returns:
        AI-generated answer with source citations
    """
    return _generate_content(
        query=query,
        model="gemini-3-flash-preview",
        max_tokens=max_tokens,
        system_prompt="Be concise and factual. When you search the web, cite your sources. Avoid speculation.",
        use_grounding=True,
    )


@mcp.tool
def ask_thinking(
    query: str,
    max_tokens: int = 8192,
) -> str:
    """
    Get answers with extended thinking for complex reasoning tasks.

    Uses Gemini 2.0 Flash Thinking model which shows its reasoning process.
    Best for multi-step problems, analysis, and complex questions.

    Note: Thinking model does not support grounding, so answers are based on
    the model's training data only.

    Args:
        query: Your question or complex problem
        max_tokens: Maximum response length (default: 8192)

    Returns:
        AI-generated answer with reasoning process
    """
    return _generate_content(
        query=query,
        model="gemini-2.0-flash-thinking-exp-01-21",
        max_tokens=max_tokens,
        system_prompt=None,  # Thinking model works best without system prompts
        use_grounding=False,  # Thinking model doesn't support grounding
    )


if __name__ == "__main__":
    # Run the server - works for both local (stdio) and cloud (HTTP)
    mcp.run()
