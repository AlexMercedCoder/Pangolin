import pytest
from click.testing import CliRunner
from unittest.mock import MagicMock, patch
from pypangolin.cli.main import cli

@pytest.fixture
def runner():
    return CliRunner()

@pytest.fixture
def mock_client():
    with patch("pypangolin.cli.admin.get_client") as mock:
        yield mock

@pytest.fixture
def mock_user_client():
    with patch("pypangolin.cli.user.get_client") as mock:
        yield mock

def test_admin_list_tenants(runner, mock_client):
    # Setup mock
    client_instance = MagicMock()
    mock_client.return_value = client_instance
    
    tenant = MagicMock()
    tenant.id = "123"
    tenant.name = "test-tenant"
    client_instance.tenants.list.return_value = [tenant]
    
    result = runner.invoke(cli, ["admin", "list-tenants"])
    
    assert result.exit_code == 0
    assert "test-tenant" in result.output
    client_instance.tenants.list.assert_called_once()

def test_user_list_catalogs(runner, mock_user_client):
    # Setup mock
    client_instance = MagicMock()
    mock_user_client.return_value = client_instance
    
    catalog = MagicMock()
    catalog.name = "my-catalog"
    catalog.catalog_type = "Local"
    client_instance.catalogs.list.return_value = [catalog]
    
    result = runner.invoke(cli, ["user", "list-catalogs"])
    
    assert result.exit_code == 0
    assert "my-catalog" in result.output
    client_instance.catalogs.list.assert_called_once()
