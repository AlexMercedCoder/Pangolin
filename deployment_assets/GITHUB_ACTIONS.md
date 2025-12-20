# GitHub Actions Workflow for Binary Builds

This document explains how the automated binary build system works using GitHub Actions.

## Overview

The workflow automatically builds Pangolin binaries for all major platforms (Linux, macOS, Windows) using GitHub's cloud runners. This eliminates the need for complex cross-compilation setups on your local machine.

## Workflow File Location

[`.github/workflows/build-binaries.yml`](../.github/workflows/build-binaries.yml)

## How It Triggers

The workflow has **two trigger mechanisms**:

### 1. Automatic: Tag Push (Recommended for Releases)

Push a git tag starting with `v` to automatically build and create a release:

```bash
# Create a version tag
git tag v0.1.0

# Push the tag to GitHub
git push origin v0.1.0
```

**What happens:**
- ✅ Builds binaries on all platforms
- ✅ Creates a GitHub Release
- ✅ Attaches all binaries to the release
- ✅ Release is named after the tag (e.g., "v0.1.0")

**Tag naming rules:**
- Must start with `v` (e.g., `v1.0.0`, `v0.2.3-beta`)
- Can include any version format after the `v`

### 2. Manual: Workflow Dispatch (For Testing)

Manually trigger the workflow from GitHub's UI:

1. Go to https://github.com/AlexMercedCoder/Pangolin/actions
2. Click "Build Binaries" in the left sidebar
3. Click "Run workflow" button
4. Select the branch to build from
5. Click "Run workflow"

**What happens:**
- ✅ Builds binaries on all platforms
- ❌ Does NOT create a release
- 📦 Artifacts available for download from the Actions page

## Build Jobs

The workflow runs **5 parallel jobs**:

### Job 1: build-linux
- **Runner**: `ubuntu-latest`
- **Platform**: Linux x86_64
- **Builds**: `pangolin_api`, `pangolin-admin`, `pangolin-user`
- **Duration**: ~40 seconds

### Job 2: build-macos-intel
- **Runner**: `macos-13` (Intel)
- **Platform**: macOS x86_64
- **Builds**: `pangolin_api`, `pangolin-admin`, `pangolin-user`
- **Duration**: ~2-3 minutes

### Job 3: build-macos-arm
- **Runner**: `macos-14` (Apple Silicon)
- **Platform**: macOS ARM64
- **Builds**: `pangolin_api`, `pangolin-admin`, `pangolin-user`
- **Duration**: ~2-3 minutes

### Job 4: build-windows
- **Runner**: `windows-latest`
- **Platform**: Windows x86_64
- **Builds**: `pangolin_api.exe`, `pangolin-admin.exe`, `pangolin-user.exe`
- **Duration**: ~2-3 minutes

### Job 5: create-release
- **Runner**: `ubuntu-latest`
- **Condition**: Only runs when triggered by a tag
- **Action**: Creates GitHub Release with all binaries
- **Duration**: ~10 seconds

## Monitoring Workflow Runs

### View Active Runs
1. Visit: https://github.com/AlexMercedCoder/Pangolin/actions
2. Click on the running workflow
3. See real-time progress of each job

### Download Artifacts
If the workflow was manually triggered (no release created):
1. Click on the completed workflow run
2. Scroll to "Artifacts" section at the bottom
3. Download platform-specific binaries

### View Releases
If triggered by a tag:
1. Visit: https://github.com/AlexMercedCoder/Pangolin/releases
2. Find the release matching your tag
3. Download binaries from the "Assets" section

## Example Workflow

### Creating a New Release

```bash
# 1. Make your code changes
git add .
git commit -m "Add new feature"
git push origin main

# 2. Create and push a version tag
git tag v0.2.0
git push origin v0.2.0

# 3. Wait for workflow to complete (~5 minutes)
# 4. Check releases page for binaries
```

### Testing Without a Release

```bash
# 1. Push your changes to a branch
git checkout -b test-build
git push origin test-build

# 2. Go to GitHub Actions UI
# 3. Manually trigger workflow on 'test-build' branch
# 4. Download artifacts from the workflow run
```

## Workflow Configuration

Key settings in the workflow file:

```yaml
on:
  push:
    tags:
      - 'v*'           # Trigger on tags like v1.0.0
  workflow_dispatch:   # Allow manual triggering
```

## Troubleshooting

### Workflow Doesn't Trigger
- **Check**: Did you push the tag? (`git push origin v0.1.0`)
- **Check**: Does the tag start with `v`?
- **Check**: Is the workflow file in `.github/workflows/`?

### Build Fails
- **Check**: Does the code compile locally? (`cargo build --release`)
- **Check**: Are all dependencies in `Cargo.toml`?
- **View**: Click on the failed job to see error logs

### Release Not Created
- **Check**: Was the workflow triggered by a tag push?
- **Note**: Manual dispatch does NOT create releases

## Cost & Limits

GitHub Actions is **free** for public repositories with generous limits:
- 2,000 minutes/month for private repos (not applicable here)
- Unlimited for public repos
- Each workflow run uses ~10-15 minutes total across all jobs

## Security

The workflow:
- ✅ Runs in isolated GitHub-managed runners
- ✅ Uses official Rust toolchain
- ✅ No secrets required for building
- ✅ Artifacts are publicly downloadable (for public repos)

## Related Documentation

- [bin/README.md](./bin/README.md) - Using pre-compiled binaries
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
