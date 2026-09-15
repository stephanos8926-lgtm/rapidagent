"""Core module for RapidAgent."""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any

logger = logging.getLogger(__name__)


@dataclass
class ProviderConfig:
    """Configuration for an LLM provider."""

    name: str
    api_key: str = ""
    base_url: str | None = None
    model: str = ""
    max_tokens: int = 4096

    @classmethod
    def from_env(cls, name: str, prefix: str) -> ProviderConfig:
        """Create provider config from environment variables."""
        import os

        api_key = os.getenv(f"{prefix}_API_KEY", "")
        base_url = os.getenv(f"{prefix}_BASE_URL")
        model = os.getenv(f"{prefix}_MODEL", "")
        max_tokens = int(os.getenv(f"{prefix}_MAX_TOKENS", "4096"))

        return cls(
            name=name,
            api_key=api_key,
            base_url=base_url,
            model=model,
            max_tokens=max_tokens,
        )


@dataclass
class AgentConfig:
    """Configuration for an Agent instance."""

    name: str = "agent"
    description: str = ""
    provider: ProviderConfig | None = None
    system_prompt: str = """\
You are {name}, a helpful AI assistant developed by RapidWebs Enterprise.
You are designed to be helpful, harmless, and honest."""
    temperature: float = 0.7
    top_p: float = 0.9
    max_iterations: int = 20

    def validate(self) -> None:
        """Validate configuration."""
        if self.provider is None:
            raise ValueError("provider is required")
        if not self.provider.api_key:
            raise ValueError(f"API key required for provider '{self.provider.name}'")
        if not self.provider.model:
            raise ValueError(f"Model required for provider '{self.provider.name}'")


class Agent:
    """
    Core agent implementation.

    The Agent is the main building block for RapidAgent.
    It handles LLM interaction, tool execution, and context management.
    """

    def __init__(self, config: AgentConfig) -> None:
        self.config = config
        self.name = config.name
        self._context: list[dict[str, Any]] = []

        config.validate()
        logger.info("Initialized agent '%s' with provider '%s'", self.name, config.provider.name)

    def chat(self, message: str) -> str:
        """
        Send a message to the agent and get a response.

        Args:
            message: The user's message

        Returns:
            The agent's response
        """
        logger.info("Chat received from user: %s", message[:100])

        self._context.append({"role": "user", "content": message})

        # TODO: Implement LLM call and tool execution
        response = "Placeholder response - implementation in progress"

        self._context.append({"role": "assistant", "content": response})

        return response

    def clear_context(self) -> None:
        """Clear the conversation context."""
        self._context.clear()
        logger.info("Cleared context for agent '%s'", self.name)

    @property
    def context(self) -> list[dict[str, Any]]:
        """Get the current conversation context."""
        return self._context.copy()

    def __repr__(self) -> str:
        return f"<Agent(name='{self.name}', provider='{self.config.provider.name}')>"
