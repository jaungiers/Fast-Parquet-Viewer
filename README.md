# Fast Parquet Viewer

A fast, lightweight desktop viewer for `.parquet` files, built with Rust and [egui](https://github.com/emilk/egui).
Has a modern interface that opens extremely fast and can be bound to .parquet files to make it the default app for opening parquet files.

Windows binary available and compiled under `target/release/ParquetViewer.exe`.

![Screenshot Dark](screenshot_dark.png)

![Screenshot Light](screenshot_light.png)

## Features

- **Drag & drop** a `.parquet` or `.parq` file onto the window to open it
- **File dialog** via the `Open…` button or `Ctrl+O`
- **Virtual scrolling** — handles large files without loading the full table into view at once
- **Column sorting** — click any column header to sort ascending/descending (numeric-aware)
- **Search / filter** — `Ctrl+F` to filter rows by any matching cell value, with match highlighting
- **Schema display** — column names and data types shown in the header
- **Status bar** — shows row/column count and file size
- **CLI support** — pass a file path as an argument: `ParquetViewer.exe data.parquet`
- Dark theme

## Download

Download the latest `ParquetViewer.exe` from the [Releases](../../releases) page. It is a single self-contained executable — no installer or additional files required.

## Building from source

Requires [Rust](https://rustup.rs/) (stable).

```sh
git clone https://github.com/jaungiers/Fast-Parquet-Viewer
cd Fast-Parquet-Viewer
cargo build --release
```

## Dependencies

| Crate | Purpose |
|---|---|
| [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / [egui](https://github.com/emilk/egui) | GUI framework |
| [egui_extras](https://github.com/emilk/egui/tree/master/crates/egui_extras) | Virtual-scroll table widget |
| [arrow](https://github.com/apache/arrow-rs) | Column data model |
| [parquet](https://github.com/apache/arrow-rs/tree/master/parquet) | Parquet file reading |
| [rfd](https://github.com/PolyMeilex/rfd) | Native file dialog |

## Author
Jakob Aungiers
