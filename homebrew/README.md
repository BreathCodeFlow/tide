# Homebrew Distribution

This directory contains the Homebrew formula for Tide.

## Setting Up Your Homebrew Tap

1. **Create a new GitHub repository** named `homebrew-tap` (the name must start with `homebrew-`):
   ```bash
   gh repo create homebrew-tap --public --description "Homebrew tap for Tide"
   ```

2. **Clone and set up the tap repository**:
   ```bash
   cd ..
   git clone https://github.com/BreathCodeFlow/homebrew-tap.git
   cd homebrew-tap
   mkdir -p Formula
   ```

3. **Copy and update the formula**:
   ```bash
   cp ../tide/homebrew/tide.rb Formula/
   ```

4. **Update the SHA256 checksums** in `Formula/tide.rb`:
   - After creating a release (e.g., v0.1.0), download the checksums:
     ```bash
     curl -sL https://github.com/BreathCodeFlow/tide/releases/download/v0.1.0/tide-aarch64-apple-darwin.tar.gz.sha256
     curl -sL https://github.com/BreathCodeFlow/tide/releases/download/v0.1.0/tide-x86_64-apple-darwin.tar.gz.sha256
     ```
   - Replace `REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64` and `REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64` with the actual values

5. **Commit and push**:
   ```bash
   git add Formula/tide.rb
   git commit -m "Add tide formula"
   git push origin main
   ```

## Installing Tide via Homebrew

Once your tap is set up, users can install Tide with:

```bash
brew tap BreathCodeFlow/tap
brew install tide
```

Or in one command:

```bash
brew install BreathCodeFlow/tap/tide
```

## Updating the Formula

When you release a new version:

1. Create a new git tag and push it:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. The GitHub Actions workflow will automatically create a release with binaries

3. Update the formula in your tap repository:
   - Update the version number
   - Update the SHA256 checksums with the new release checksums
   - Commit and push the changes

## Testing the Formula Locally

Before publishing, test the formula:

```bash
brew install --build-from-source Formula/tide.rb
brew test tide
brew audit --strict tide
```
