# vegam


Jerk-Limited TOPP

```
pip install pyvegam
```

## Development

```bash
uv sync --group test   # builds the C++ extension and installs it in .venv
uv run pytest

cargo test             # builds the C++ library and runs the Rust tests
```
