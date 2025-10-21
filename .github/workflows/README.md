# GitHub Actions Setup

## Release Workflow

Der Release-Workflow wird automatisch ausgelöst, wenn ein neuer Tag (z.B. `v1.2.0`) gepusht wird.

### Automatisierte Schritte:

1. **Build für beide Architekturen** (ARM64 und x86_64)
2. **Erstellen von Release-Archives** (.tar.gz)
3. **Berechnen der SHA256-Checksums**
4. **Upload der Release-Assets zu GitHub**
5. **Automatisches Update der Homebrew Formula**

### Erforderliches Secret einrichten:

Um die Homebrew Formula automatisch zu aktualisieren, muss ein GitHub Token erstellt werden:

#### 1. Personal Access Token erstellen:

1. Gehe zu: https://github.com/settings/tokens
2. Klicke auf "Generate new token" → "Generate new token (classic)"
3. Name: `Homebrew Tap Update`
4. Setze folgende Berechtigungen:
   - ✅ `repo` (Full control of private repositories)
5. Klicke auf "Generate token"
6. **Kopiere den Token** (wird nur einmal angezeigt!)

#### 2. Secret im tide Repository hinzufügen:

1. Gehe zu: https://github.com/BreathCodeFlow/tide/settings/secrets/actions
2. Klicke auf "New repository secret"
3. Name: `HOMEBREW_TAP_TOKEN`
4. Value: [Den kopierten Token einfügen]
5. Klicke auf "Add secret"

### Neues Release erstellen:

```bash
# 1. Version in Cargo.toml aktualisieren
# 2. Änderungen committen
git add Cargo.toml
git commit -m "chore: bump version to 1.3.0"

# 3. Tag erstellen
git tag -a v1.3.0 -m "Release v1.3.0

- Feature 1
- Feature 2
- Fix 1
"

# 4. Tag pushen (triggert automatisch den Release-Workflow)
git push && git push --tags
```

### Was passiert automatisch:

1. GitHub Actions baut die Binaries für beide Architekturen
2. Erstellt Release-Archives und berechnet SHA256-Checksums
3. Lädt alles als GitHub Release hoch
4. Klont das homebrew-tap Repository
5. Aktualisiert die Formula mit neuer Version und SHA256-Checksums
6. Committed und pusht die Änderungen ins homebrew-tap Repository

### Manueller Fallback:

Falls die automatische Aktualisierung fehlschlägt, kann die Homebrew Formula manuell aktualisiert werden:

```bash
# SHA256 aus Release-Assets abrufen
curl -L -o tide-aarch64.tar.gz https://github.com/BreathCodeFlow/tide/releases/download/v1.2.0/tide-aarch64-apple-darwin.tar.gz
curl -L -o tide-x86_64.tar.gz https://github.com/BreathCodeFlow/tide/releases/download/v1.2.0/tide-x86_64-apple-darwin.tar.gz

shasum -a 256 tide-aarch64.tar.gz
shasum -a 256 tide-x86_64.tar.gz

# homebrew-tap klonen und Formula aktualisieren
git clone https://github.com/BreathCodeFlow/homebrew-tap.git
cd homebrew-tap
# Formula/tide.rb bearbeiten
git add Formula/tide.rb
git commit -m "chore: update tide formula to v1.2.0"
git push
```
