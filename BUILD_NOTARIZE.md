# macOS App Signing & Notarization Guide

This guide explains how to build and notarize the EXP Tracker app for macOS distribution.

## Prerequisites

### 1. Apple Developer Account
- Paid Apple Developer membership ($99/year)
- https://developer.apple.com

### 2. Code Signing Certificate
Install "Developer ID Application" certificate:

```bash
# List available signing identities
security find-identity -v -p codesigning
```

You should see something like:
```
1) XXXXXXXXXX "Developer ID Application: Your Name (TEAM_ID)"
```

### 3. App-Specific Password (Method 1)
Generate at: https://appleid.apple.com/account/manage

1. Sign in to your Apple ID account
2. Go to "Security" section
3. Click "Generate Password" under "App-Specific Passwords"
4. Save the generated password (format: `xxxx-xxxx-xxxx-xxxx`)

### 4. API Key (Method 2 - Recommended)
Generate at: https://appstoreconnect.apple.com/access/api

1. Go to "Keys" tab
2. Click "+" to create new key
3. Give it "Developer" access
4. Download the `.p8` file (only available once!)
5. Note the Key ID and Issuer ID

## Setup

### 1. Create Environment File
```bash
cp .env.example .env
```

### 2. Fill in Credentials

#### Option A: Using App-Specific Password
```bash
# .env
APPLE_ID=your-apple-id@example.com
APPLE_PASSWORD=xxxx-xxxx-xxxx-xxxx
APPLE_TEAM_ID=XXXXXXXXXX
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (XXXXXXXXXX)"
```

#### Option B: Using API Key (Recommended)
```bash
# .env
APPLE_API_KEY_PATH=/path/to/AuthKey_XXXXXXXXXX.p8
APPLE_API_KEY_ID=XXXXXXXXXX
APPLE_API_ISSUER_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
APPLE_TEAM_ID=XXXXXXXXXX
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (XXXXXXXXXX)"
```

### 3. Load Environment Variables
```bash
# Load credentials before building
source .env
export APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID APPLE_SIGNING_IDENTITY
# or for API Key method:
export APPLE_API_KEY_PATH APPLE_API_KEY_ID APPLE_API_ISSUER_ID APPLE_TEAM_ID APPLE_SIGNING_IDENTITY
```

## Building

### Regular Build (No Signing/Notarization)
Fast build for development testing:
```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/macos/exp-tracker-temp.app`
- Not signed
- Will show "untrusted developer" warning
- For testing only

### Notarized Build (Signed + Notarized)
Production-ready build:
```bash
npm run build:notarize
```

Output: `src-tauri/target/release/bundle/macos/exp-tracker-temp.app`
- Code signed with Developer ID
- Notarized by Apple
- No warnings on user's Mac
- Ready for distribution

## Build Process Details

### What `npm run build:notarize` does:
1. Builds Python OCR server (`scripts/build_python_server.sh`)
2. Generates app icons
3. Builds frontend (TypeScript + Vite)
4. Compiles Rust backend
5. **Signs** the app with your Developer ID
6. **Uploads** to Apple Notary Service
7. **Staples** notarization ticket to the app

### Notarization Time
- First time: 5-15 minutes
- Subsequent builds: 2-5 minutes

You'll see output like:
```
🔐 Signing application...
📤 Uploading to Apple Notary Service...
⏳ Waiting for notarization (this may take several minutes)...
✅ Notarization successful!
📎 Stapling notarization ticket...
✅ Build complete: exp-tracker-temp.app
```

## Verifying Notarization

### Check Code Signing
```bash
codesign -dvv src-tauri/target/release/bundle/macos/exp-tracker-temp.app
```

Should show:
```
Authority=Developer ID Application: Your Name (TEAM_ID)
Signed Time=...
```

### Check Notarization
```bash
spctl -a -vv src-tauri/target/release/bundle/macos/exp-tracker-temp.app
```

Should show:
```
exp-tracker-temp.app: accepted
source=Notarized Developer ID
```

## Troubleshooting

### "No signing identity found"
```bash
# Install your Developer ID certificate from Xcode
# or download from developer.apple.com
security find-identity -v -p codesigning
```

### "Authentication failed"
- Check APPLE_ID and APPLE_PASSWORD are correct
- Ensure you're using App-Specific Password, not regular password
- For API Key: verify file path and IDs are correct

### "Invalid entitlements"
- Check `src-tauri/entitlements.plist` syntax
- Ensure required entitlements match app capabilities

### "Notarization failed"
```bash
# Check notarization log
xcrun notarytool log <request-id> --apple-id YOUR_APPLE_ID
```

## Distribution

### DMG Creation (Optional)
```bash
npm install --save-dev electron-installer-dmg
# Create DMG from .app
```

### Distribution Checklist
- ✅ App is signed with Developer ID
- ✅ App is notarized by Apple
- ✅ Notarization ticket is stapled
- ✅ App launches without warnings
- ✅ Test on fresh Mac (no developer tools)

## Security Notes

### DO NOT commit:
- `.env` file (contains credentials)
- `.p8` API Key files
- Code signing certificates

### Keep secure:
- App-Specific Passwords
- API Keys and certificates
- Use environment variables or CI/CD secrets

## CI/CD Integration

For automated builds (GitHub Actions, etc.):

```yaml
- name: Setup signing
  env:
    APPLE_API_KEY: ${{ secrets.APPLE_API_KEY }}
    APPLE_API_KEY_ID: ${{ secrets.APPLE_API_KEY_ID }}
    APPLE_API_ISSUER_ID: ${{ secrets.APPLE_API_ISSUER_ID }}
  run: |
    echo "$APPLE_API_KEY" > AuthKey.p8
    export APPLE_API_KEY_PATH="$(pwd)/AuthKey.p8"
    npm run build:notarize
```

## References

- [Apple Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Tauri Bundle Documentation](https://tauri.app/v1/guides/building/)
- [Code Signing Guide](https://developer.apple.com/support/code-signing/)
