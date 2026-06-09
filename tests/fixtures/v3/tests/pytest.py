# Pytest test fixtures
import pytest

def test_simple():
    assert 1 + 1 == 2

class TestUserService:
    def test_get_user(self):
        assert get_user("1").id == "1"

    def test_create_user(self):
        assert create_user("test").name == "test"

@pytest.fixture
def db_session():
    session = create_session()
    yield session
    session.close()

@pytest.mark.slow
def test_slow_operation():
    assert True
