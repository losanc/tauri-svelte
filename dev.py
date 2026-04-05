#!/usr/bin/env python3
"""
dev.py — build / test / deploy helper
Usage: python dev.py <command> [options]
"""

import sys
import subprocess
import argparse
from tabnanny import check


# ── helpers ────────────────────────────────────────────────────────────────────
def run(cmd: list[str], *, check: bool = True, capture: bool = False) -> subprocess.CompletedProcess:
    print(f"  » {' '.join(cmd)}")
    return subprocess.run(cmd, check=check, capture_output=capture, text=True)

def git(*args: str, capture: bool = False) -> subprocess.CompletedProcess:
    return run(["git", *args], capture=capture)


def current_branch() -> str:
    result = git("rev-parse", "--abbrev-ref", "HEAD", capture=True)
    return result.stdout.strip()

def cmd_git_set_tag(tag: str):
    run(["git", "tag", "-d", tag], check=False)
    run(["git", "push", "origin", f":refs/tags/{tag}"], check=False)
    git("tag", tag)
    git("push", "origin", tag)


# ── commands ───────────────────────────────────────────────────────────────────
def cmd_tag_test(args):
    """Remove any existing 'test' tag (local + remote) and re-create it at HEAD."""
    branch = current_branch()
    print(f"\n[tag:test] branch={branch}")
    cmd_git_set_tag("test")

def cmd_tag_release(args):
    """Remove any existing 'release' tag (local + remote) and re-create it at HEAD."""
    branch = current_branch()
    print(f"\n[tag:release] branch={branch}")
    cmd_git_set_tag("release")

def cmd_run_test(args):
    """Runs the rs & js tests."""
    branch = current_branch()
    print(f"\n[tag:test] branch={branch}")
    run(["pnpm", "check"], check=False)
    run(["pnpm", "test"], check=False)
    run(["cargo", "fmt", "--all", "--", "--check"], check=False)
    run(["cargo", "test", "--workspace"], check=False)

def cmd_run_fmt(args):
    """Runs the rs & js formatting."""
    branch = current_branch()
    print(f"\n[tag:fmt] branch={branch}")
    run(["pnpm", "format"], check=False)
    run(["cargo", "fmt", "--all"], check=False)


# ── CLI ─────────────────────────────────────────────────────────────────
COMMANDS = {
    "test": (cmd_run_test, "Runs the rs & js tests"),
    "fmt": (cmd_run_fmt, "Runs the rs & js formatting"),
    "tag:test": (cmd_tag_test, "Reset and push the 'test' tag to the current HEAD"),
    "tag:release": (cmd_tag_release, "Reset and push the 'release' tag to the current HEAD"),
}

def main():
    parser = argparse.ArgumentParser(
        prog="dev.py",
        description="Build / test / deploy helper",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="Commands:\n" + "\n".join(f"  {k:<14} {v[1]}" for k, v in COMMANDS.items()),
    )
    parser.add_argument("command", choices=COMMANDS, help="Command to run")

    # parse only the first positional so subcommands can add their own args later
    args, remaining = parser.parse_known_args()

    fn, _ = COMMANDS[args.command]
    try:
        fn(args)
    except subprocess.CalledProcessError as e:
        print(f"\n✗  Command failed (exit {e.returncode}): {' '.join(e.cmd)}", file=sys.stderr)
        sys.exit(e.returncode)
    except KeyboardInterrupt:
        print("\nAborted.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()