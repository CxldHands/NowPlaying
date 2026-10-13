const API_URL = "/now-playing";
const SEEK_THRESHOLD_MS = 250;

// ─────────────────────────────────────────────
// UI State
// ─────────────────────────────────────────────

const currentState = {
    title: null,
    artist: null,
    artworkVersion: null,
    artworkUrl: null,
    artworkRetryAt: 0,
    artworkRetryDelay: 1000,
    isPlaying: false,
    

    // Local playback clock.
    positionMs: 0,
    durationMs: 0,
    positionAnchorTime: 0,
};

// Prevent concurrent polling requests.
let isUpdating = false;
let progressFrameId = null;

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
    if (!data.artwork_version || data.artwork_version === 0) {
        return null;
    }

    try {
        const response = await fetch(
            `/artwork?v=${data.artwork_version}`
        );

        if (!response.ok) {
            throw new Error(
                `Artwork API returned HTTP ${response.status}`
            );
        }

        const blob = await response.blob();

        if (!blob.type.startsWith("image/") || blob.size === 0) {
            throw new Error("Artwork response is not a valid image");
        }

        currentState.artworkRetryDelay = 1000;
        currentState.artworkRetryAt = 0;

        return URL.createObjectURL(blob);
    } catch (error) {
        console.error("Could not load artwork:", error);

        currentState.artworkRetryAt =
            performance.now() + currentState.artworkRetryDelay;

        // Back off gradually, up to 30 seconds.
        currentState.artworkRetryDelay = Math.min(
            currentState.artworkRetryDelay * 2,
            30000
        );

        return null;
    }
}

function replaceArtworkUrl(newUrl) {
    if (currentState.artworkUrl) {
        URL.revokeObjectURL(currentState.artworkUrl);
    }

    currentState.artworkUrl = newUrl;
}

// ─────────────────────────────────────────────
// Playback State
// ─────────────────────────────────────────────

function updatePlaybackState(data) {
    const now = performance.now();
    const reportedPosition = Math.max(0, data.position_ms);
    const reportedDuration = Math.max(0, data.duration_ms);
    const isPlaying = Boolean(data.is_playing);

    // Predict playback position using the local clock.
    const elapsed = currentState.isPlaying
        ? now - currentState.positionAnchorTime
        : 0;

    const predictedPosition = currentState.positionMs + elapsed;
    const trackChanged = hasTrackChanged(data);
    const playbackChanged = isPlaying !== currentState.isPlaying;
    const positionDifference = Math.abs(reportedPosition - predictedPosition);

    // Resynchronize after a track change, playback-state change, or significant position correction such as a seek.
    const shouldResync = trackChanged || playbackChanged || positionDifference > SEEK_THRESHOLD_MS;

    currentState.durationMs = reportedDuration;

    if (shouldResync) {
        currentState.positionMs = reportedPosition;
    } else {
        currentState.positionMs = predictedPosition;
    }

    currentState.positionAnchorTime = now;
    currentState.isPlaying = isPlaying;

    if (isPlaying) {
        startProgressAnimation();
    } else {
        stopProgressAnimation();

        skin.updateProgress({
            position_ms: currentState.positionMs,
            duration_ms: currentState.durationMs,
        });
    }
}

// ─────────────────────────────────────────────
// Progress Animation
// ─────────────────────────────────────────────

function startProgressAnimation() {
    if (progressFrameId !== null) {
        return;
    }

    progressFrameId = requestAnimationFrame(renderProgressFrame);
}

function stopProgressAnimation() {
    if (progressFrameId !== null) {
        cancelAnimationFrame(progressFrameId);
        progressFrameId = null;
    }
}

function renderProgressFrame() {
    progressFrameId = null;

    if (!currentState.isPlaying) {
        return;
    }

    const elapsed = performance.now() - currentState.positionAnchorTime;

    let positionMs = currentState.positionMs + elapsed;

    if (currentState.durationMs > 0) {
        positionMs = Math.min(
            positionMs,
            currentState.durationMs
        );
    }

    skin.updateProgress({
        position_ms: positionMs,
        duration_ms: currentState.durationMs,
    });

    progressFrameId = requestAnimationFrame(renderProgressFrame);
}

// ─────────────────────────────────────────────
// Main Update
// ─────────────────────────────────────────────

async function updateNowPlaying() {
    // Skip a poll if the previous request is still running.
    if (isUpdating) {
        return;
    }

    isUpdating = true;

    try {
        const response = await fetch(API_URL);

        if (!response.ok) {
            throw new Error(
                `NowPlaying API returned HTTP ${response.status}`
            );
        }

        const data = await response.json();
        const trackChanged = hasTrackChanged(data);

        const artworkChanged = data.artwork_version !== currentState.artworkVersion;

        const hasArtwork = Boolean(data.title && data.artwork_version);
        const shouldRetryArtwork = hasArtwork && !currentState.artworkUrl && performance.now() >= currentState.artworkRetryAt;

        if (trackChanged || artworkChanged || shouldRetryArtwork) {
            let artworkUrl = currentState.artworkUrl;

            if (!hasArtwork) {
                artworkUrl = null;
                currentState.artworkRetryAt = 0;
                currentState.artworkRetryDelay = 1000;
            } else if (artworkChanged || shouldRetryArtwork) {
                const fetchedArtwork = await fetchArtwork(data);

                if (fetchedArtwork) {
                    artworkUrl = fetchedArtwork;
                }
            }

            skin.updateTrack(
                data,
                artworkUrl,
                Boolean(data.title)
            );

            if (artworkUrl !== currentState.artworkUrl) {
                replaceArtworkUrl(artworkUrl);
            }

            currentState.title = data.title;
            currentState.artist = data.artist;
            currentState.artworkVersion = data.artwork_version;
        }

        updatePlaybackState(data);

    } catch (error) {
        console.error("Could not update NowPlaying:", error);
    } finally {
        isUpdating = false;
    }
}

// ─────────────────────────────────────────────
// Cleanup
// ─────────────────────────────────────────────

window.addEventListener("pagehide", () => {
    stopProgressAnimation();

    if (currentState.artworkUrl) {
        URL.revokeObjectURL(currentState.artworkUrl);
        currentState.artworkUrl = null;
    }
});

// ─────────────────────────────────────────────
// Initialization
// ─────────────────────────────────────────────

updateNowPlaying();

// Keep the UI synchronized with Rust.
setInterval(updateNowPlaying, 1000);