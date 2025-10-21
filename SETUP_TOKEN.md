# ⚠️ Wichtig: GitHub Secret einrichten

Damit die automatische Homebrew Formula Aktualisierung funktioniert, muss noch ein GitHub Token erstellt werden:

## Schnell-Anleitung:

### 1. Token erstellen
```
https://github.com/settings/tokens/new
```

- **Name:** `Homebrew Tap Update`
- **Berechtigung:** ✅ `repo` (volle Kontrolle)
- Token kopieren!

### 2. Secret hinzufügen
```
https://github.com/BreathCodeFlow/tide/settings/secrets/actions/new
```

- **Name:** `HOMEBREW_TAP_TOKEN`
- **Value:** [Token einfügen]

## Danach funktioniert:

```bash
git tag -a v1.3.0 -m "Release v1.3.0"
git push --tags
```

→ Automatisch:
- ✅ Builds erstellen
- ✅ Release auf GitHub
- ✅ Homebrew Formula aktualisiert
- ✅ SHA256-Checksums berechnet

---

**Vollständige Dokumentation:** [.github/workflows/README.md](.github/workflows/README.md)
