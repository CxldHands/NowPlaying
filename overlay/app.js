const API_URL = "/now-playing";

// ─────────────────────────────────────────────
// UI State
// ─────────────────────────────────────────────

// This represents what the application currently knows.
// We compare incoming data against this state
// so we don't unnecessarily update the UI.

const currentState = {
    title: null,
    artist: null,
    artworkVersion: null,
    artworkUrl: null,
    isPlaying: false,
};

// ─────────────────────────────────────────────
// Track State
// ─────────────────────────────────────────────

function hasTrackChanged(data) {
    return (
        data.title !== currentState.title ||
        data.artist !== currentState.artist
    );
}

// ─────────────────────────────────────────────
// Artwork
// ─────────────────────────────────────────────

async function fetchArtwork(data) {
    if (!data.artwork_version) {
        return null;
    }

    try {
        const response = await fetch(
            `/artwork?v=${data.artwork_version}`
        );

        if (!response.ok) {
            return null;
        }

        return URL.createObjectURL(await response.blob());
    } catch (error) {
        console.error("Could not load artwork:", error);
        return null;
    }
}

// ─────────────────────────────────────────────
// Playback State
// ─────────────────────────────────────────────

function updatePlaybackState(data) {
    if (data.is_playing === currentState.isPlaying) {
        return;
    }

    currentState.isPlaying = data.is_playing;

}

// ─────────────────────────────────────────────
// Main Update
// ─────────────────────────────────────────────

async function updateNowPlaying() {
    try {
        const response = await fetch(API_URL);
        const data = await response.json();

        if (hasTrackChanged(data)) {
            const artworkUrl = await fetchArtwork(data);

            skin.updateTrack(
                data,
                artworkUrl,
                Boolean(data.title)
            );

            if (artworkUrl) {
                if (currentState.artworkUrl) {
                    URL.revokeObjectURL(currentState.artworkUrl);
                }

                currentState.artworkUrl = artworkUrl;
            }

            currentState.title = data.title;
            currentState.artist = data.artist;
            currentState.artworkVersion = data.artwork_version;
        }

        updatePlaybackState(data);
        skin.updateProgress(data);

    } catch (error) {
        console.error(
            "Could not connect to NowPlaying:",
            error
        );
    }
}

// ─────────────────────────────────────────────
// Initialization
// ─────────────────────────────────────────────

updateNowPlaying();

// Keep the UI synchronized with Rust.
setInterval(updateNowPlaying, 1000);