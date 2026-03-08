#!/usr/bin/env python3
"""Backward-compatible wrapper for the canonical gh_issue_run CLI."""

from gh_issue_run import main


if __name__ == "__main__":
    raise SystemExit(main())
