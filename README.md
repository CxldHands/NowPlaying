# NowPlaying v0.2.0
NowPlaying is a lightweight Windows app that reads currently playing media through Windows Media Session / System Media Transport Controls (SMTC) and provides a browser-based overlay for OBS.


## Requirements
- Windows
- Spotify
- OBS Studio (if you want to use the overlay)


## Getting Started
1. Extract the NowPlaying release package.
2. Run `NowPlaying.exe`.
3. NowPlaying will run in the Windows system tray.
4. Open the NowPlaying control page from the tray menu.
5. Start playing music in Spotify.

The control page lets you:

- Choose an overlay skin.
- Preview the selected skin.
- Copy or reveal the OBS overlay URL.
- Open the overlay directly in your browser.
- See the recommended Browser Source dimensions.
- Check Spotify/media connection status.


## Adding NowPlaying to OBS
In OBS:

1. Add a **Browser Source**.
2. Copy the overlay URL from the NowPlaying control page.
3. Paste the URL into the Browser Source URL field.
4. Set the Browser Source dimensions to the recommended size shown by NowPlaying.


## Overlay Skins
### Default
A vertical album-art-focused layout with track information and playback progress.
Recommended Browser Source size for the DEFAULT skin - **400 × 520 px**

### Ambient
A compact horizontal layout with album artwork, track information, playback progress, and blurred artwork in the background.
Recommended Browser Source size for the AMBIENT skin - **470 × 140 px**


## Spotify
NowPlaying currently detects Spotify through Windows Media Session / SMTC.
The app does not require Spotify API credentials or direct access to your Spotify account.

## System Tray
NowPlaying runs in the Windows system tray while it is active.
Use the tray menu to:

- Open NowPlaying
- Quit NowPlaying

## Version
**v0.2.0**

See `CHANGELOG.md` for the changes included in this release.
