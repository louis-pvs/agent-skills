#!/usr/bin/env python3
"""Self-Progress Retrospective Script (ADR 0004 path safety, stdlib-first).

Parses transcript JSONL log files from an agent session to extract
capability gaps, error patterns, user corrections, and research activities.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

# Dynamic sys.path resolution (Repo Rule)
_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._path_safety import sanitize_path
except ImportError:

    def sanitize_path(input_path: Any, base_dir: Optional[Path] = None, *, strict_chars: bool = False) -> Path:
        p = Path(input_path).resolve()
        return p


def parse_transcript(transcript_path: Path) -> List[Dict[str, Any]]:
    """Safely parse a transcript JSONL file line by line.

    Args:
        transcript_path: Path to transcript JSONL file.

    Returns:
        List of step dictionary records.
    """
    records: List[Dict[str, Any]] = []
    if not transcript_path.exists():
        return records

    with open(transcript_path, encoding="utf-8") as f:
        for line in f:
            line_str = line.strip()
            if not line_str:
                continue
            try:
                data = json.loads(line_str)
                if isinstance(data, dict):
                    records.append(data)
            except json.JSONDecodeError:
                continue
    return records


def extract_signals(records: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    """Extract retrospective gap signals from parsed transcript records.

    Signals extracted:
    - errors: tool execution failures or step errors
    - user_corrections: user messages following an error or rejected step
    - research_patterns: web search / documentation lookups indicating missing knowledge
    - repeated_struggles: repeated tool execution on the same target

    Args:
        records: Parsed transcript records.

    Returns:
        Dictionary mapping signal categories to lists of signal details.
    """
    errors: List[Dict[str, Any]] = []
    user_corrections: List[Dict[str, Any]] = []
    research_patterns: List[Dict[str, Any]] = []
    repeated_struggles: List[Dict[str, Any]] = []

    last_was_error = False
    tool_counts: Dict[str, int] = {}

    for record in records:
        step_type = record.get("type", "")
        status = record.get("status", "")
        content = record.get("content", "")
        tool_calls = record.get("tool_calls", [])

        # 1. Error signals
        if status == "ERROR" or (isinstance(content, str) and "Error" in content and step_type == "PLANNER_RESPONSE"):
            for tc in tool_calls:
                errors.append(
                    {
                        "step_index": record.get("step_index"),
                        "tool_name": tc.get("name", "unknown"),
                        "args": tc.get("args", {}),
                        "content_snippet": str(content)[:200] if content else "",
                    }
                )
            if not tool_calls and status == "ERROR":
                errors.append(
                    {
                        "step_index": record.get("step_index"),
                        "tool_name": "N/A",
                        "args": {},
                        "content_snippet": str(content)[:200] if content else "",
                    }
                )
            last_was_error = True
        else:
            # 2. User correction signals
            if last_was_error and step_type == "USER_INPUT":
                user_corrections.append(
                    {
                        "step_index": record.get("step_index"),
                        "content": str(content),
                    }
                )
            last_was_error = False

        # 3. Research patterns (missing agent knowledge)
        for tc in tool_calls:
            tname = tc.get("name", "")
            tool_counts[tname] = tool_counts.get(tname, 0) + 1

            if tname in ("search_web", "read_url_content", "search_pages"):
                args = tc.get("args", {})
                query = args.get("query") or args.get("Url") or args.get("Query") or ""
                research_patterns.append(
                    {
                        "step_index": record.get("step_index"),
                        "tool_name": tname,
                        "query": query,
                    }
                )

    # 4. Repeated struggles (same tool invoked > 5 times in a session)
    for tname, count in tool_counts.items():
        if count >= 5 and tname not in ("view_file", "list_dir"):
            repeated_struggles.append({"tool_name": tname, "invocation_count": count})

    return {
        "errors": errors,
        "user_corrections": user_corrections,
        "research_patterns": research_patterns,
        "repeated_struggles": repeated_struggles,
    }


def format_markdown_report(signals: Dict[str, List[Dict[str, Any]]]) -> str:
    """Format extracted signals as a clean Markdown report.

    Args:
        signals: Signal dictionary from extract_signals.

    Returns:
        Markdown report string.
    """
    lines = ["# 🔄 Self-Progress Conversation Retrospective\n"]

    errors = signals.get("errors", [])
    corrections = signals.get("user_corrections", [])
    research = signals.get("research_patterns", [])
    struggles = signals.get("repeated_struggles", [])

    lines.append("## Summary of Signals\n")
    lines.append(f"- **Errors / Failed Tool Executions**: {len(errors)}")
    lines.append(f"- **User Corrections Detected**: {len(corrections)}")
    lines.append(f"- **Research / Knowledge Searches**: {len(research)}")
    lines.append(f"- **High-Frequency Tool Chains**: {len(struggles)}\n")

    if errors:
        lines.append("### 🔴 Errors & Tool Failures")
        for err in errors:
            lines.append(f"- Step {err.get('step_index')}: `{err.get('tool_name')}` — {err.get('content_snippet')}")
        lines.append("")

    if corrections:
        lines.append("### 🟡 User Corrections")
        for corr in corrections:
            lines.append(f"- Step {corr.get('step_index')}: {corr.get('content')}")
        lines.append("")

    if research:
        lines.append("### 🔵 Research & Missing Knowledge Queries")
        for r in research:
            lines.append(f"- Step {r.get('step_index')}: Tool `{r.get('tool_name')}` — Query: `{r.get('query')}`")
        lines.append("")

    if struggles:
        lines.append("### 🟠 High-Frequency Tool Usage")
        for s in struggles:
            lines.append(f"- Tool `{s.get('tool_name')}` invoked {s.get('invocation_count')} times.")
        lines.append("")

    if not any([errors, corrections, research, struggles]):
        lines.append("✨ No capability gaps or error signals detected in transcript. Session ran cleanly.")

    return "\n".join(lines)


def parse_args():
    parser = argparse.ArgumentParser(description="Self-Progress Retrospective Transcript Analyzer")
    parser.add_argument("--transcript", type=str, required=True, help="Path to transcript.jsonl file")
    parser.add_argument("--json", action="store_true", help="Output raw JSON format")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    try:
        transcript_path = sanitize_path(args.transcript, base_dir=Path.cwd())
    except ValueError as err:
        sys.stderr.write(f"Error resolving path: {err}\n")
        return 1

    records = parse_transcript(transcript_path)
    signals = extract_signals(records)

    if args.json:
        print(json.dumps(signals, indent=2))
    else:
        print(format_markdown_report(signals))

    return 0


if __name__ == "__main__":
    sys.exit(main())
