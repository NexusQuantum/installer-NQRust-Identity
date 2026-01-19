# NexusQuantum Identity Installer

An interactive Terminal User Interface (TUI) installer for deploying the NQRust Identity stack (Keycloak + PostgreSQL) using Docker Compose.

## Overview

This installer provides a guided setup experience for the NQRust Identity platform, which includes:
- **PostgreSQL 16** - Database backend for Keycloak
- **Keycloak** - Open-source Identity and Access Management solution

The installer handles:
- GitHub Container Registry (GHCR) authentication
- Docker Compose orchestration
- Service health checks
- One-click deployment

## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/NexusQuantum/installer-NQRust-Identity/main/scripts/install/install.sh | bash
```

Installs the latest `.deb` from GitHub Releases and makes `nqrust-identity` available in `$PATH`. Then run:

```bash
nqrust-identity
```

### Manual Installation

1) Download the latest `.deb` from the [Releases](https://github.com/NexusQuantum/installer-NQRust-Identity/releases) page:

```bash
curl -LO https://github.com/NexusQuantum/installer-NQRust-Identity/releases/latest/download/nqrust-identity_amd64.deb
```

2) Install the package:

```bash
sudo apt install ./nqrust-identity_amd64.deb
# or: sudo dpkg -i nqrust-identity_amd64.deb
```

3) Run the installer:

```bash
nqrust-identity
```

### Build from Source

```bash
git clone https://github.com/NexusQuantum/installer-NQRust-Identity.git
cd installer-NQRust-Identity
cargo build --release
./target/release/nqrust-identity
```

## Usage

### Interactive TUI

Simply run:

```bash
nqrust-identity
```

The installer will guide you through:

1. **Registry Setup** (Optional) - Authenticate with GitHub Container Registry
   - Provide your GitHub Personal Access Token (PAT)
   - Token needs `read:packages` scope
   - Skip this step if using public images

2. **Confirmation** - Review services to be deployed
   - PostgreSQL 16 Alpine
   - Keycloak (NQRust Identity)

3. **Installation** - Automated deployment
   - Pulls Docker images
   - Starts services via Docker Compose
   - Monitors deployment progress

4. **Success** - Access your Keycloak instance
   - Admin Console: http://localhost:8080
   - Default credentials: admin / admin
   - **⚠️ Change password after first login!**

### Default Configuration

The installer deploys with these defaults:

**Keycloak:**
- Port: `8080`
- Admin username: `admin`
- Admin password: `admin`
- Database: `identity`
- Theme: `keycloakify-starter`

**PostgreSQL:**
- Port: `5432`
- Database: `identity`
- Username: `identity`
- Password: `identity`

### Customization

To customize the deployment, edit `docker-compose.yaml` before running the installer:

```yaml
services:
  identity:
    environment:
      KEYCLOAK_ADMIN: your-admin-username
      KEYCLOAK_ADMIN_PASSWORD: your-secure-password
      KC_DB_USERNAME: your-db-username
      KC_DB_PASSWORD: your-db-password
```

Or set environment variables:

```bash
export KEYCLOAK_ADMIN=myadmin
export KEYCLOAK_ADMIN_PASSWORD=mysecurepassword
nqrust-identity
```

## Post-Installation

### Access Keycloak

1. Open browser: http://localhost:8080
2. Login with default credentials: `admin` / `admin`
3. **Immediately change the admin password**
4. Configure your realm and clients

### Manage Services

```bash
# Check service status
docker compose ps

# View logs
docker compose logs -f identity
docker compose logs -f postgres

# Stop services
docker compose down

# Restart services
docker compose up -d

# Remove all data (⚠️ destructive)
docker compose down -v
```

### Update Services

Use the installer's built-in update checker:

```bash
nqrust-identity
# Navigate to "Check for updates"
```

Or manually:

```bash
docker compose pull
docker compose up -d
```

## Troubleshooting

### Docker Login Issues

If GHCR authentication fails:

```bash
# Test manual login
echo "YOUR_PAT" | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Check Docker is running
docker info

# Verify PAT has correct scopes
# Go to: https://github.com/settings/tokens
# Required: read:packages
```

### Port Conflicts

If port 8080 is already in use:

```bash
# Edit docker-compose.yaml
# Change: "8080:8080" to "8081:8080"

# Or set environment variable
export KEYCLOAK_PORT=8081
```

### Database Connection Issues

```bash
# Check PostgreSQL logs
docker compose logs postgres

# Verify database is ready
docker compose exec postgres pg_isready -U identity

# Reset database (⚠️ destructive)
docker compose down -v
docker compose up -d
```

## Requirements

- **OS**: Linux (Ubuntu 20.04+, Debian 11+, or compatible)
- **Docker**: 20.10+ with Docker Compose v2
- **Architecture**: x86_64/amd64
- **Disk Space**: ~500MB for images
- **RAM**: 2GB minimum, 4GB recommended

## Security Notes

1. **Change default passwords immediately**
2. Use strong passwords in production
3. Enable HTTPS for production deployments
4. Restrict database access
5. Keep Keycloak and PostgreSQL updated
6. Review Keycloak security best practices

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build .deb package
cargo install cargo-deb
cargo deb
```

### Testing

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Project Structure

```
installer-NQRust-Identity/
├── src/
│   ├── main.rs              # Entry point
│   ├── app/                 # Application logic
│   │   ├── mod.rs          # Main app state machine
│   │   ├── state.rs        # State definitions
│   │   ├── registry_form.rs # GHCR auth form
│   │   └── updates.rs      # Update checker
│   ├── ui/                  # TUI components
│   │   ├── confirmation.rs
│   │   ├── installing.rs
│   │   ├── success.rs
│   │   └── ...
│   └── utils.rs            # Utilities
├── docker-compose.yaml      # Service definitions
├── scripts/
│   └── install/
│       └── install.sh      # One-liner installer
└── Cargo.toml              # Project metadata
```

## Support

- GitHub Issues: [NexusQuantum/installer-NQRust-Identity](https://github.com/NexusQuantum/installer-NQRust-Identity/issues)
- Documentation: [Keycloak Docs](https://www.keycloak.org/documentation)

## License

See LICENSE file for details.
