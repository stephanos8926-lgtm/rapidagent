"""Command-line interface for the project."""

import argparse
import sys

from .core import hello


def main() -> int:
    """Main entry point for the CLI."""
    parser = argparse.ArgumentParser(
        prog="[PROJECT_NAME]",
        description="[PROJECT_NAME] CLI tool",
    )
    parser.add_argument(
        "--name",
        default="World",
        help="Name to greet (default: World)",
    )
    parser.add_argument(
        "--version",
        action="version",
        version="%(prog)s 0.1.0",
    )

    args = parser.parse_args()
    print(hello(args.name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
