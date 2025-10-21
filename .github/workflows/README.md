# GitHub Actions Setup

## Release Workflow

The release workflow is automatically triggered when a new tag (e.g., `v1.2.0`) is pushed.

### Automated Steps:

1. **Build for both architectures** (ARM64 and x86_64)
2. **Create release archives** (.tar.gz)
3. **Calculate SHA256 checksums**
4. **Upload release assets to GitHub**
5. **Automatically update Homebrew Formula**

### Required Secret Setup:

To automatically update the Homebrew Formula, a GitHub Token needs to be created:

#### 1. Create Personal Access Token:

1. Go to: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Name: `Homebrew Tap Update`
4. Set the following permissions:
   - ✅ `repo` (Full control of private repositories)
5. Click "Generate token"
6. **Copy the token** (shown only once!)

#### 2. Add secret to the tide repository:

1. Go to: https://github.com/BreathCodeFlow/tide/settings/secrets/actions
2. Click "New repository secret"
3. Name: `HOMEBREW_TAP_TOKEN`
4. Value: [Paste the copied token]
5. Click "Add secret"

### Create a new release:

```bash
# 1. Update version in Cargo.toml
# 2. Commit changes
git add Cargo.toml
git commit -m "chore: bump version to 1.3.0"

# 3. Create tag
git tag -a v1.3.0 -m "Release v1.3.0

- Feature 1
- Feature 2
- Fix 1
"

# 4. Push tag (automatically triggers the release workflow)
git push && git push --tags
```

### What happens automatically:

1. GitHub Actions builds binaries for both architectures
2. Creates release archives and calculates SHA256 checksums
3. Uploads everything as a GitHub Release
4. Clones the homebrew-tap repository
5. Updates the Formula with the new version and SHA256 checksums
6. Commits and pushes the changes to the homebrew-tap repository

### Manual Fallback:

If the automatic update fails, the Homebrew Formula can be updated manually:

```bash
# Get SHA256 from release assets
curl -L -o tide-aarch64.tar.gz https://github.com/BreathCodeFlow/tide/releases/download/v1.2.0/tide-aarch64-apple-darwin.tar.gz
curl -L -o tide-x86_64.tar.gz https://github.com/BreathCodeFlow/tide/releases/download/v1.2.0/tide-x86_64-apple-darwin.tar.gz

shasum -a 256 tide-aarch64.tar.gz
shasum -a 256 tide-x86_64.tar.gz

# Clone homebrew-tap and update formula
git clone https://github.com/BreathCodeFlow/homebrew-tap.git
cd homebrew-tap
# Edit Formula/tide.rb
git add Formula/tide.rb
git commit -m "chore: update tide formula to v1.2.0"
git push
```
