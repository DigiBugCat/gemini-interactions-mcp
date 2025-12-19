"""
Gemini Interactions MCP Server

A FastMCP server using the Gemini Interactions API for stateful,
grounded AI conversations with Google Search.

Features:
- Stateful conversations via interaction_id
- Auto-grounding (model decides when to search)
- Multiple thinking levels (minimal/low/medium/high)
- File upload support
"""

import os
from typing import Optional, Literal
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
INTERACTIONS_ENDPOINT = "https://generativelanguage.googleapis.com/v1beta/interactions"
MODEL = "gemini-3-flash-preview"


def _create_interaction(
    input_content: str | list,
    thinking_level: Literal["minimal", "low", "medium", "high"] = "medium",
    previous_interaction_id: Optional[str] = None,
    max_tokens: int = 8192,
    system_instruction: Optional[str] = None,
) -> dict:
    """
    Create an interaction with the Gemini API.

    Returns parsed response with text, sources, interaction_id, and usage.
    """
    payload = {
        "model": MODEL,
        "input": input_content,
        "store": True,  # Enable caching
        "generation_config": {
            "thinking_level": thinking_level,
            "max_output_tokens": max_tokens,
        },
        # Always include grounding tools - model auto-decides when to use them
        "tools": [
            {"type": "google_search"},
            {"type": "url_context"}
        ]
    }

    if previous_interaction_id:
        payload["previous_interaction_id"] = previous_interaction_id

    if system_instruction:
        payload["system_instruction"] = system_instruction

    headers = {
        "x-goog-api-key": GEMINI_API_KEY,
        "Content-Type": "application/json"
    }

    try:
        with httpx.Client(timeout=120.0) as client:
            response = client.post(INTERACTIONS_ENDPOINT, json=payload, headers=headers)
            response.raise_for_status()
            data = response.json()

        return _parse_interaction_response(data)

    except httpx.HTTPStatusError as e:
        return {
            "error": f"API error: {e.response.status_code} - {e.response.text}",
            "interaction_id": None,
            "status": "failed"
        }
    except Exception as e:
        return {
            "error": f"Request failed: {str(e)}",
            "interaction_id": None,
            "status": "failed"
        }


def _parse_interaction_response(data: dict) -> dict:
    """Parse the interaction response into a structured format."""
    result = {
        "interaction_id": data.get("id"),
        "status": data.get("status"),
        "text": "",
        "sources": [],
        "usage": data.get("usage", {})
    }

    for output in data.get("outputs", []):
        output_type = output.get("type")

        if output_type == "text":
            result["text"] += output.get("text", "")
            # Extract annotations as inline citations
            for ann in output.get("annotations", []):
                source = ann.get("source")
                if source and source not in result["sources"]:
                    result["sources"].append(source)

        elif output_type == "google_search_result":
            for item in output.get("result", []):
                source = {
                    "url": item.get("url"),
                    "title": item.get("title")
                }
                if source["url"] and source not in result["sources"]:
                    result["sources"].append(source)

        elif output_type == "url_context_result":
            for item in output.get("result", []):
                if item.get("status") == "success":
                    source = {
                        "url": item.get("url"),
                        "title": "URL Context"
                    }
                    if source["url"] and source not in result["sources"]:
                        result["sources"].append(source)

    return result


def _format_response(result: dict) -> str:
    """Format the parsed result into a readable string."""
    if "error" in result:
        return f"Error: {result['error']}"

    output = [result.get("text", "")]

    # Add sources
    sources = result.get("sources", [])
    if sources:
        output.append("\n\nSources:")
        for i, source in enumerate(sources, 1):
            if isinstance(source, dict):
                title = source.get("title", "Untitled")
                url = source.get("url", "")
                output.append(f"{i}. [{title}]({url})")
            else:
                output.append(f"{i}. {source}")

    # Add follow-up instructions
    output.append("\n---")
    output.append(f"To follow up, use interaction_id: {result.get('interaction_id', 'N/A')}")

    return "\n".join(output)


# MCP Tools

@mcp.tool
def search(
    query: str,
    max_results: int = 10,
) -> str:
    """
    Quick web search with minimal thinking. Returns structured results.

    Use this to find sources before asking follow-up questions.

    Args:
        query: Search query
        max_results: Maximum number of results to return (default: 10)

    Returns:
        Structured search results with titles, URLs, and snippets
    """
    system_instruction = f"""Search for the query and return results in this exact format:

---
TITLE: [page title]
URL: [full url]
SNIPPET: [2-3 sentence excerpt]
---

Return up to {max_results} results. No additional commentary or analysis."""

    result = _create_interaction(
        input_content=query,
        thinking_level="minimal",
        system_instruction=system_instruction,
        max_tokens=4096,
    )

    return _format_response(result)


@mcp.tool
def ask(
    query: str,
    interaction_id: Optional[str] = None,
    max_tokens: int = 8192,
) -> str:
    """
    Get grounded answers with balanced reasoning.

    Model automatically searches the web when needed for current information.
    To follow up on a previous response, pass the interaction_id from that response.

    Args:
        query: Your question
        interaction_id: Pass the interaction_id from a previous response to continue that conversation
        max_tokens: Maximum response length (default: 8192)

    Returns:
        Answer with sources. Use the returned interaction_id to ask follow-up questions.
    """
    result = _create_interaction(
        input_content=query,
        thinking_level="medium",
        previous_interaction_id=interaction_id,
        max_tokens=max_tokens,
        system_instruction="Be concise and factual. Cite sources when using web information.",
    )

    return _format_response(result)


@mcp.tool
def ask_thinking(
    query: str,
    interaction_id: Optional[str] = None,
    max_tokens: int = 16384,
) -> str:
    """
    Get answers with deep reasoning for complex problems.

    Uses high thinking level for multi-step analysis and complex questions.
    To follow up on a previous response, pass the interaction_id from that response.

    Args:
        query: Your complex question or problem
        interaction_id: Pass the interaction_id from a previous response to continue that conversation
        max_tokens: Maximum response length (default: 16384)

    Returns:
        Detailed answer with reasoning. Use the returned interaction_id to ask follow-up questions.
    """
    result = _create_interaction(
        input_content=query,
        thinking_level="high",
        previous_interaction_id=interaction_id,
        max_tokens=max_tokens,
        system_instruction="Think step by step. Be thorough and cite sources.",
    )

    return _format_response(result)


if __name__ == "__main__":
    mcp.run()
