import os
from .settings import CONFIG


def hello() -> str:
    return CONFIG.get("message", "hello from api")
