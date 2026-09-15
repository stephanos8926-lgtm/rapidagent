"""Tests for the RapidAgent core module."""

import pytest

from rapidagent.core import Agent, AgentConfig, ProviderConfig


def test_agent_creation():
    """Test basic agent creation."""
    config = AgentConfig(
        name="test-agent",
        provider=ProviderConfig(
            name="test",
            api_key="dummy",
            model="test-model",
        ),
    )
    agent = Agent(config)
    assert agent.name == "test-agent"
    assert str(agent) == "<Agent(name='test-agent', provider='test')>"


def test_agent_chat():
    """Test basic chat functionality."""
    config = AgentConfig(
        name="test-agent",
        provider=ProviderConfig(
            name="test",
            api_key="dummy",
            model="test-model",
        ),
    )
    agent = Agent(config)
    response = agent.chat("Hello")
    assert isinstance(response, str)
    assert len(agent.context) == 2


def test_agent_clear_context():
    """Test context clearing."""
    config = AgentConfig(
        name="test-agent",
        provider=ProviderConfig(
            name="test",
            api_key="dummy",
            model="test-model",
        ),
    )
    agent = Agent(config)
    agent.chat("Hello")
    assert len(agent.context) == 2

    agent.clear_context()
    assert len(agent.context) == 0


def test_provider_from_env(monkeypatch):
    """Test provider config from environment."""
    monkeypatch.setenv("TEST_API_KEY", "test-key")
    monkeypatch.setenv("TEST_MODEL", "test-model")
    monkeypatch.setenv("TEST_MAX_TOKENS", "8192")

    provider = ProviderConfig.from_env("test", "TEST")
    assert provider.name == "test"
    assert provider.api_key == "test-key"
    assert provider.model == "test-model"
    assert provider.max_tokens == 8192


def test_config_validation_missing_provider():
    """Test validation fails without provider."""
    config = AgentConfig(name="test")
    with pytest.raises(ValueError, match="provider is required"):
        config.validate()


def test_config_validation_missing_api_key():
    """Test validation fails without API key."""
    config = AgentConfig(
        name="test",
        provider=ProviderConfig(name="test", model="model"),
    )
    with pytest.raises(ValueError, match="API key required"):
        config.validate()


def test_config_validation_missing_model():
    """Test validation fails without model."""
    config = AgentConfig(
        name="test",
        provider=ProviderConfig(name="test", api_key="key"),
    )
    with pytest.raises(ValueError, match="Model required"):
        config.validate()
