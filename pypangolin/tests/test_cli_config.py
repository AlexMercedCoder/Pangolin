import pytest
from pypangolin.cli.config import get_active_profile, set_active_profile, load_config, save_config, CONFIG_FILE

def test_load_default_config(tmp_path, monkeypatch):
    monkeypatch.setattr("pypangolin.cli.config.CONFIG_FILE", tmp_path / "profiles.yaml")
    config = load_config()
    assert config["active_profile"] == "default"
    assert config["profiles"]["default"]["username"] == "admin"

def test_save_and_load_config(tmp_path, monkeypatch):
    monkeypatch.setattr("pypangolin.cli.config.CONFIG_FILE", tmp_path / "profiles.yaml")
    config = {
        "active_profile": "test",
        "profiles": {
            "test": {"url": "http://test", "username": "user"}
        }
    }
    save_config(config)
    
    loaded = load_config()
    assert loaded["active_profile"] == "test"
    assert loaded["profiles"]["test"]["url"] == "http://test"

def test_set_active_profile(tmp_path, monkeypatch):
    monkeypatch.setattr("pypangolin.cli.config.CONFIG_FILE", tmp_path / "profiles.yaml")
    config = {
        "active_profile": "default",
        "profiles": {
            "default": {},
            "test": {}
        }
    }
    save_config(config)
    
    set_active_profile("test")
    loaded = load_config()
    assert loaded["active_profile"] == "test"
