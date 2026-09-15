"""Pytest configuration for New Project Template."""

import sys
from pathlib import Path

# Add src directory to path so tests can import new_project
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))
