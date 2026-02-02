# VidGenie

A video generator that creates MP4 videos from JSON configuration files. Built with Rust, OpenGL, and GStreamer.

## Features

- Generate videos from JSON timeline definitions
- Support for image assets (local files, URLs, file:// URIs)
- Configurable output resolution and format
- Image positioning, scaling, and rotation
- Fade transitions between clips
- Transform keyframe animations

## Setup

### Using Docker (Recommended)

Build the Docker image which includes all dependencies:

```bash
make docker-build
```

### Native Installation

#### Requirements

- Rust (stable toolchain)
- GStreamer 1.x with development libraries
- OpenGL/EGL development libraries

#### macOS

```bash
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly
```

#### Ubuntu/Debian

```bash
sudo apt-get install -y \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-gl \
    libegl1-mesa-dev \
    libgbm-dev
```

#### Arch Linux

```bash
sudo pacman -S gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly
```

## Usage

### Render with Docker

```bash
# List available examples
make list-examples

# Render a specific example
make docker-render-example EXAMPLE=local_files.json

# Output will be saved to outputs/local_files.mp4
```

### Render Natively

```bash
# Build the project
make build

# Render an example
make render-example EXAMPLE=local_files.json

# Or run directly
cargo run -p vg-cli -- --file examples/local_files.json --output output.mp4
```

### JSON Configuration Format

```json
{
    "output": {
        "format": "mp4",
        "width": 1280,
        "height": 720
    },
    "timeline": {
        "background": "#101418",
        "tracks": [
            {
                "clips": [
                    {
                        "asset": {
                            "type": "image",
                            "src": "./path/to/image.jpg"
                        },
                        "start": 0.0,
                        "length": 3.0,
                        "offset": { "x": 0, "y": 0 },
                        "scale": 1.0,
                        "position": "center"
                    }
                ]
            }
        ]
    }
}
```

#### Asset Sources

- Relative path: `./resources/image.jpg`
- Absolute path: `/path/to/image.jpg`
- File URI: `file:///path/to/image.jpg`
- HTTP URL: `https://example.com/image.jpg`

#### Clip Properties

| Property | Type | Description |
|----------|------|-------------|
| `start` | float | Start time in seconds |
| `length` | float | Duration in seconds |
| `offset` | object | Position offset `{x, y}` in pixels |
| `scale` | float | Scale factor (1.0 = original size) |
| `rotate` | float | Rotation in degrees |
| `position` | string | Positioning mode (`"center"`) |
| `transition` | object | Fade transition settings |

## Development

### Available Make Commands

```bash
make help           # Show all available commands
make build          # Build the project
make fmt            # Format code
make lint           # Run linter
make unit-test      # Run tests
make clean          # Clean build artifacts
```

### Project Structure

```
├── src/
│   ├── vg-cli/      # Command-line interface
│   ├── vg-gl/       # OpenGL rendering
│   ├── vg-gst/      # GStreamer video encoding
│   └── vg-video/    # Video/timeline logic
├── examples/        # Example JSON configurations
├── dockerfiles/     # Docker build files
└── outputs/         # Generated videos (gitignored)
```

### Running Tests

```bash
make unit-test
```

### Code Formatting

```bash
make fmt
```

## License

MIT
