# Chronicle

A Rust CLI tool that generates daily chronicles from your local development activity. Track Git commits, TODO changes, and notes in one consolidated daily log.

## What is Chronicle?

Chronicle automatically monitors your local development signals and generates human-readable daily summaries:

- **Git Activity**: Commits, branches, and file changes across multiple repositories
- **TODO Tracking**: Changes to your TODO and Inbox files (new items, completed items, modifications)
- **Notes**: New and modified notes from your note-taking directories

Each chronicle shows only what changed since the last run, making it easy to review your daily progress.

## Installation

### Prerequisites

- Rust 1.70 or later
- Git (for repository tracking)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/alexruf/chronicle.git
cd chronicle

# Build the project
cargo build --release

# The binary will be available at target/release/chronicle
# Optionally, install it to your system
cargo install --path .
```

## Quick Start

### 1. Initialize Configuration

```bash
chronicle config init
```

This creates a `chronicle.toml` configuration file in your current directory.

### 2. Configure Your Sources

Edit `chronicle.toml` to point to your repositories, TODO files, and notes:

```toml
output_dir = "chronicles"
state_file = ".chronicle-state.json"

[[repos]]
path = "/path/to/your/project"

[[repos]]
path = "/path/to/another/project"

todo_files = [
    "~/Documents/TODO.md",
    "~/Documents/Inbox.txt",
]

notes_dirs = [
    "~/Documents/notes",
]

[limits]
max_commits = 50
max_changed_files = 80
max_note_files = 30
max_chars_per_item = 2000

[display]
show_authors = true
```

See [`chronicle.toml.example`](chronicle.toml.example) for a complete configuration reference.

### 3. Generate Your First Chronicle

```bash
chronicle gen
```

This creates a chronicle for today in `chronicles/chronicle-YYYY-MM-DD.md`.

### 4. View the Latest Chronicle

```bash
chronicle show latest
```

## Usage

### Generate Chronicles

```bash
# Generate chronicle for today
chronicle gen

# Generate for a specific date
chronicle gen --date 2024-01-15

# Generate for a date range
chronicle gen --since 2024-01-10

# Show only specific sources
chronicle gen --only git
chronicle gen --only todos,notes

# Preview without writing to file
chronicle gen --dry-run
```

### View Chronicles

```bash
# Show the most recent chronicle
chronicle show latest
```

### Manage State

Chronicle tracks state to show only new changes since the last run. To reset:

```bash
chronicle state reset
```

## How It Works

### State Tracking

Chronicle maintains a `.chronicle-state.json` file that tracks:
- Last seen commit per Git branch
- TODO item hashes and their previous states
- Note file modification times

This allows Chronicle to show only what's changed since the last time you ran it.

### Chronicle Format

Each generated chronicle includes:

1. **Summary Table**: Quick overview of activity across all sources
2. **Git Repositories**: Commits grouped by repository and branch, with change indicators (NEW, MODIFIED)
3. **TODO Items**: Changes with status (NEW, DONE, MODIFIED) and previous state
4. **Notes**: New and modified notes with excerpts

### Example Output

```markdown
# Chronicle: 2024-01-15

Generated: 2024-01-15 18:30:00
Period: Since 2024-01-14 18:00:00

## Summary
| Source | Count |
|--------|-------|
| Repositories | 2 |
| Total Commits | 5 |
| TODO Items | 3 |
| Notes | 2 |

## Git Activity

### my-project (NEW branch: feature/auth)
- **feature/auth** (NEW, 3 commits ahead)
  - abc1234 - Add user authentication (John Doe)
    Files: src/auth.rs, src/models/user.rs
  ...
```

## Configuration Reference

See [`chronicle.toml.example`](chronicle.toml.example) for detailed configuration options including:

- Repository paths
- TODO file locations
- Notes directory tracking
- Output limits and formatting
- Display preferences

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No linter warnings (`cargo clippy`)
