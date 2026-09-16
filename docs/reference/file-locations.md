# File locations

The exact application-data path depends on the operating system. Use Settings → Storage to see the resolved paths for the current installation.

## Application data

The app data area contains the local vault and supporting files, including:

- encrypted SQLite database;
- keystore and password/passkey wrapping material;
- copied source PDFs under the PDF storage area;
- optional local model assets;
- runtime manifests and backup staging data.

The app data directory is not a source-control location. Back it up through the app rather than copying individual database files while the vault is active.

## Repository locations

| Path                         | Purpose                                                                      |
| ---------------------------- | ---------------------------------------------------------------------------- |
| `src/`                       | Svelte frontend, routes, stores, and UI components.                          |
| `src-tauri/`                 | Rust commands, database, authentication, ingestion, and Tauri configuration. |
| `src-tauri/binaries/`        | Target-matched native PDFium assets when supplied locally.                   |
| `ontology/`                  | Seed ontology data bundled with the application.                             |
| `docs/`                      | This MkDocs site.                                                            |
| `mkdocs.yml`                 | Documentation theme, navigation, search, and site metadata.                  |
| `.github/workflows/ci.yml`   | Hosted checks and build matrix.                                              |
| `.github/workflows/docs.yml` | MkDocs build and GitHub Pages deployment.                                    |

Never commit real reports, vault databases, passkey material, or private model assets.
