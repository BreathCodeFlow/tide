# ⚠️ Important: Setup GitHub Secret

For automatic Homebrew Formula updates to work, a GitHub Token needs to be created:

## Quick Guide:

### 1. Create Token

```
https://github.com/settings/tokens/new
```

- **Name:** `Homebrew Tap Update`
- **Permission:** ✅ `repo` (full control)
- Copy the token!

### 2. Add Secret

```
https://github.com/BreathCodeFlow/tide/settings/secrets/actions/new
```

- **Name:** `HOMEBREW_TAP_TOKEN`
- **Value:** [Paste token]

## Then it works:

```bash
git tag -a v1.3.0 -m "Release v1.3.0"
git push --tags
```

→ Automatically:

- ✅ Create builds
- ✅ Release on GitHub
- ✅ Update Homebrew Formula
- ✅ Calculate SHA256 checksums

---

**Full Documentation:** [.github/workflows/README.md](.github/workflows/README.md)
