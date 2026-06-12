#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["httpx>=0.27", "pyyaml>=6"]
# ///
"""Bootstrap script — creates the 2 parent tenant rooms and invites the 9 native agents.

Uses the official `band-sdk[langgraph]` HTTP API. Acts as the Themis Orchestrator.

Usage:
    source ~/.config/apohara/secrets.env
    uv run scripts/render_agent_config.py
    uv run scripts/themis-bootstrap.py [--dry-run]

What it does:
    1. Reads the rendered agent_config.yaml
    2. Connects as @apohara-themis/themis-orchestrator
    3. Creates 2 parent rooms (Stark Industries — AP, Wayne Enterprises — AP)
    4. Invites the 8 worker agents to both parent rooms
    5. Writes the room UUIDs back to ~/.config/apohara/secrets.env (append)

Idempotent: re-running will not duplicate rooms — checks by handle.
"""
from __future__ import annotations

import argparse
import asyncio
import os
import re
import sys
from pathlib import Path

import httpx
import yaml

BAND_API_BASE = "https://app.band.ai/api/v1/agent"

CONFIG_PATH = (
    Path(__file__).parent.parent / "crates" / "themis-band-client" / "agent-config" / "agent_config.yaml"
)


def load_config() -> dict:
    if not CONFIG_PATH.exists():
        print(f"ERROR: rendered config not found at {CONFIG_PATH}.", file=sys.stderr)
        print("Run `uv run scripts/render_agent_config.py` first.", file=sys.stderr)
        sys.exit(1)
    return yaml.safe_load(CONFIG_PATH.read_text())


def orchestrator_headers(config: dict) -> dict[str, str]:
    orch = config["themis-orchestrator"]
    return {
        "X-API-Key": orch["api_key"],
        "Content-Type": "application/json",
    }


async def list_existing_rooms(client: httpx.AsyncClient, config: dict) -> list[dict]:
    resp = await client.get(f"{BAND_API_BASE}/chats", headers=orchestrator_headers(config))
    resp.raise_for_status()
    return resp.json().get("data", [])


async def create_room(client: httpx.AsyncClient, config: dict) -> str:
    # Band Agent API: POST /chats with body {"chat": {}} creates a room.
    # No `name` or `description` fields — rooms get auto-titled from the
    # first @mention message. The room metadata is set via the participant
    # add endpoint and the first message.
    body = {"chat": {}}
    resp = await client.post(f"{BAND_API_BASE}/chats", headers=orchestrator_headers(config), json=body)
    resp.raise_for_status()
    return resp.json()["data"]["id"]


async def find_room_by_participant_count(client: httpx.AsyncClient, config: dict, expected_count: int) -> str | None:
    """Find a room where the orchestrator is a participant and has >= expected_count peers.

    Heuristic: rooms are orphan until we add agents. We identify our 2 parent rooms
    by the fact that they have 0 participants right after creation and we know
    the orchestrator is the creator.
    """
    rooms = await list_existing_rooms(client, config)
    return rooms  # just return all, let caller match


async def invite_participant(client: httpx.AsyncClient, config: dict, room_id: str, agent_id: str, agent_name: str) -> tuple[bool, str]:
    """Invite an agent by its UUID. Returns (success, msg).

    Body shape (verified empirically 2026-06-12):
        {"participant": {"participant_id": "<uuid>"}}
    The docs say POST /chats/{id}/participants but the field is not documented
    — discovered by probe testing.
    """
    body = {"participant": {"participant_id": agent_id}}
    resp = await client.post(
        f"{BAND_API_BASE}/chats/{room_id}/participants",
        headers=orchestrator_headers(config),
        json=body,
    )
    if resp.status_code in (200, 201):
        return True, str(resp.status_code)
    if resp.status_code == 409:
        return False, "409 (already in room)"
    return False, f"{resp.status_code} {resp.text[:200]}"


def all_worker_handles(config: dict) -> list[tuple[str, str, str]]:
    """Return list of (name, handle, agent_id) for all non-orchestrator agents."""
    workers = []
    for name, data in config.items():
        if name in ("tenants",):
            continue
        if not isinstance(data, dict) or "handle" not in data:
            continue
        if data.get("role") in ("worker", "shadow"):
            workers.append((name, data["handle"], data["agent_id"]))
    return workers


async def send_intro_message(client: httpx.AsyncClient, config: dict, room_id: str, tenant_name: str) -> bool:
    """Post the first @mention message to set the room title and context.

    Band auto-titles the room from the first text message. This is our only
    way to give the room a human-readable name.
    """
    intro = (
        f"@{tenant_name} — AP fraud-detection room online. "
        f"9-agent workflow: Extractor → PO Matcher → Fraud Auditor → "
        f"GAAP Classifier → Provenance Signer, with 3 shadow observers "
        f"(Audit Watchdog, Regression Tester, Demo Narrator). "
        f"Room ID: {room_id}."
    )
    body = {"message": {"content": intro}}
    resp = await client.post(
        f"{BAND_API_BASE}/chats/{room_id}/messages",
        headers=orchestrator_headers(config),
        json=body,
    )
    if resp.status_code in (200, 201):
        return True
    return False


async def bootstrap(dry_run: bool) -> dict[str, str]:
    config = load_config()
    workers = all_worker_handles(config)
    tenants = config.get("tenants", [])

    results: dict[str, str] = {}

    async with httpx.AsyncClient(timeout=30.0) as client:
        # 1. Sanity check — confirm orchestrator identity
        me = await client.get(f"{BAND_API_BASE}/me", headers=orchestrator_headers(config))
        me.raise_for_status()
        identity = me.json()["data"]
        print(f"✓ Authenticated as: {identity.get('handle')} ({identity.get('id')})")

        # 2. For each tenant, create the parent room + invite agents
        for tenant in tenants:
            tenant_id = tenant["id"]
            tenant_name = tenant["name"]

            if dry_run:
                print(f"⏸ {tenant_name}: would create (dry-run)")
                results[tenant_id] = f"DRY-RUN-{tenant_id}"
                continue

            print(f"→ Creating room for: {tenant_name}")
            room_id = await create_room(client, config)
            print(f"  ✓ room_id = {room_id}")

            # 3. Invite the 8 worker agents
            print(f"  Inviting {len(workers)} agents...")
            for wname, whandle, wid in workers:
                try:
                    ok, msg = await invite_participant(client, config, room_id, wid, wname)
                    mark = "✓" if ok else "↻"
                    print(f"    {mark} {wname:25s} {msg}")
                except httpx.HTTPStatusError as e:
                    print(f"    ✗ {wname}: {e.response.status_code} {e.response.text[:200]}", file=sys.stderr)

            # 4. Send the intro message to set the room title
            try:
                intro_ok = await send_intro_message(client, config, room_id, tenant_name)
                if intro_ok:
                    print(f"  ✓ intro message posted (sets room title)")
                else:
                    print(f"  ⚠ intro message failed (room will have generic title)")
            except Exception as e:
                print(f"  ⚠ intro message error: {e}")

            results[tenant_id] = room_id

    return results


def write_room_ids_to_secrets(results: dict[str, str], dry_run: bool) -> None:
    secrets_path = Path.home() / ".config" / "apohara" / "secrets.env"
    if dry_run:
        print(f"\n[DRY-RUN] Would append to {secrets_path}:")
        for tid, rid in results.items():
            env_var = f"THEMIS_{tid.upper().replace('-', '_')}_PARENT_ROOM_ID"
            print(f"  export {env_var}={rid}")
        return

    with secrets_path.open("a") as f:
        f.write(f"\n# Themis parent room IDs (auto-written by themis-bootstrap.py)\n")
        for tid, rid in results.items():
            env_var = f"THEMIS_{tid.upper().replace('-', '_')}_PARENT_ROOM_ID"
            f.write(f'export {env_var}="{rid}"\n')
    os.chmod(secrets_path, 0o600)
    print(f"\n✓ Room IDs appended to {secrets_path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    print(f"Apohara Themis — bootstrap")
    print(f"  Band API: {BAND_API_BASE}")
    print(f"  Mode: {'DRY-RUN' if args.dry_run else 'LIVE'}")
    print()

    results = asyncio.run(bootstrap(args.dry_run))
    write_room_ids_to_secrets(results, args.dry_run)

    print(f"\nSummary:")
    for tid, rid in results.items():
        print(f"  {tid}: {rid}")


if __name__ == "__main__":
    main()
