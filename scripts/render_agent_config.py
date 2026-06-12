#!/usr/bin/env python3
"""Render agent_config.yaml.tmpl → agent_config.yaml by expanding ${ENV_VAR} placeholders.

Usage:
    source ~/.config/apohara/secrets.env
    uv run scripts/render_agent_config.py

NEVER commit the rendered file. The template uses ${VAR} placeholders that
the Rust themis-band-client reads at runtime, so this script is optional —
the Rust side can also expand on the fly. This script exists for debugging
and for the Python bootstrap.
"""
from __future__ import annotations

import os
import re
import sys
from pathlib import Path

TEMPLATE_PATH = Path(__file__).parent.parent / "crates" / "themis-band-client" / "agent-config" / "agent_config.yaml.tmpl"
OUTPUT_PATH = TEMPLATE_PATH.parent / "agent_config.yaml"

PLACEHOLDER_RE = re.compile(r"\$\{([A-Z_][A-Z0-9_]*)\}")


def expand(match: re.Match[str]) -> str:
    var_name = match.group(1)
    value = os.environ.get(var_name)
    if value is None:
        print(f"ERROR: env var {var_name} is not set. Did you source secrets.env?", file=sys.stderr)
        sys.exit(1)
    return value


def main() -> None:
    template = TEMPLATE_PATH.read_text()
    rendered = PLACEHOLDER_RE.sub(expand, template)
    OUTPUT_PATH.write_text(rendered)
    # chmod 600 because the file now contains real API keys
    os.chmod(OUTPUT_PATH, 0o600)
    print(f"Rendered: {OUTPUT_PATH}")
    print(f"Mode: 600 (owner-only). Do NOT commit this file.")


if __name__ == "__main__":
    main()
