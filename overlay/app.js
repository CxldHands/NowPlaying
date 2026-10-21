const API_URL = "/now-playing";
const SEEK_THRESHOLD_MS = 250;
const POLL_INTERVAL_MS = 1000;
const API_FAILURE_THRESHOLD = 3;

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

let isUpdating = false;
let progressFrameId = null;
let artworkRequestId = 0;
let pollingIntervalId = null;
let pageLifecycleId = 0;

let apiFailureCount = 0;
let apiUnavailable = false;
let hasInitializedTrackState = false;

// ─────────────────────────────────────────────
// Track State
// ─────────────────────────────────────────────

function hasTrackChanged(data) {
    return (
        data.title !== currentState.title ||
        data.artist !== currentState.artist
    );
}

function isValidNowPlayingData(data) {
    if (!data || typeof data !== "object" || Array.isArray(data)) {
        return false;
    }

    const validText = (value) =>
        value === null || typeof value === "string";

    const validNumber = (value) =>
        typeof value === "number" && Number.isFinite(value);

    const validArtworkVersion = (value) =>
        value === null || validNumber(value);

    return (
        validText(data.title) &&
        validText(data.artist) &&
        validArtworkVersion(data.artwork_version) &&
        validNumber(data.position_ms) &&
        validNumber(data.duration_ms) &&
        typeof data.is_playing === "boolean"
    );
}

// ─────────────────────────────────────────────
// Artwork
// ─────────────────────────────────────────────

async function fetchArtwork(data) {
    if (
        !data.artwork_version ||
        data.artwork_version === 0
    ) {
        return null;
    }

    const requestId = ++artworkRequestId;
    const requestedVersion = data.artwork_version;

    try {
        const response = await fetch(
            `/artwork?v=${requestedVersion}`
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

        if (requestId !== artworkRequestId) {
            return null;
        }

        currentState.artworkRetryDelay = 1000;
        currentState.artworkRetryAt = 0;

        return URL.createObjectURL(blob);
    } catch (error) {
        if (requestId !== artworkRequestId) {
            return null;
        }

        console.error("Could not load artwork:", error);

        currentState.artworkRetryAt =
            performance.now() + currentState.artworkRetryDelay;

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

function resetArtworkRetry() {
    currentState.artworkRetryAt = 0;
    currentState.artworkRetryDelay = 1000;
}

// ─────────────────────────────────────────────
// Playback State
// ─────────────────────────────────────────────

function updatePlaybackState(data, trackChanged) {
    const now = performance.now();
    const reportedPosition = Math.max(0, data.position_ms);
    const reportedDuration = Math.max(0, data.duration_ms);
    const isPlaying = Boolean(data.is_playing);

    const elapsed = currentState.isPlaying
        ? now - currentState.positionAnchorTime
        : 0;

    const predictedPosition = currentState.positionMs + elapsed;
    const playbackChanged = isPlaying !== currentState.isPlaying;
    const positionDifference = Math.abs(
        reportedPosition - predictedPosition
    );

    const shouldResync =
        trackChanged ||
        playbackChanged ||
        positionDifference > SEEK_THRESHOLD_MS;

    currentState.durationMs = reportedDuration;
    currentState.positionMs = shouldResync
        ? reportedPosition
        : predictedPosition;

    currentState.positionAnchorTime = now;
    currentState.isPlaying = isPlaying;

    if (typeof skin.updatePlaybackState === "function") {
        skin.updatePlaybackState({
            is_playing: isPlaying,
        });
    }

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

    const elapsed =
        performance.now() - currentState.positionAnchorTime;

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
// Unavailable State
// ─────────────────────────────────────────────

function clearTrackState() {
    artworkRequestId++;

    replaceArtworkUrl(null);
    resetArtworkRetry();

    currentState.title = null;
    currentState.artist = null;
    currentState.artworkVersion = null;
    currentState.isPlaying = false;
    currentState.positionMs = 0;
    currentState.durationMs = 0;
    currentState.positionAnchorTime = performance.now();

    stopProgressAnimation();

    skin.updateTrack(
        {
            title: null,
            artist: null,
            artwork_version: null,
            position_ms: 0,
            duration_ms: 0,
            is_playing: false,
        },
        null,
        false
    );

    if (typeof skin.updatePlaybackState === "function") {
        skin.updatePlaybackState({
            is_playing: false,
        });
    }

    skin.updateProgress({
        position_ms: 0,
        duration_ms: 0,
    });

    hasInitializedTrackState = true;
}

function handleApiFailure(error) {
    apiFailureCount++;

    console.error(
        `Could not update NowPlaying ` +
        `(${apiFailureCount}/${API_FAILURE_THRESHOLD}):`,
        error
    );

    if (
        apiFailureCount >= API_FAILURE_THRESHOLD &&
        !apiUnavailable
    ) {
        apiUnavailable = true;
        clearTrackState();
    }
}

// ─────────────────────────────────────────────
// Main Update
// ─────────────────────────────────────────────

async function updateNowPlaying() {
    if (isUpdating || pollingIntervalId === null) {
        return;
    }

    isUpdating = true;

    const requestLifecycleId = pageLifecycleId;

    try {
        const response = await fetch(API_URL);

        if (!response.ok) {
            throw new Error(
                `NowPlaying API returned HTTP ${response.status}`
            );
        }

        const data = await response.json();

        if (requestLifecycleId !== pageLifecycleId) {
            return;
        }

        if (!isValidNowPlayingData(data)) {
            throw new Error("NowPlaying API returned invalid data");
        }

        apiFailureCount = 0;
        apiUnavailable = false;

        const trackChanged = hasTrackChanged(data);

        const artworkChanged =
            data.artwork_version !== currentState.artworkVersion;

        const hasTrack = Boolean(data.title);
        const hasArtwork = Boolean(
            hasTrack && data.artwork_version
        );

        const now = performance.now();

        const shouldAttemptArtwork =
            hasArtwork &&
            (artworkChanged || !currentState.artworkUrl) &&
            now >= currentState.artworkRetryAt;

        const shouldUpdateTrack =
            !hasInitializedTrackState ||
            trackChanged ||
            (!hasArtwork && artworkChanged) ||
            shouldAttemptArtwork;

        if (shouldUpdateTrack) {
            // Do not reuse artwork from an older version.
            let artworkUrl = artworkChanged
                ? null
                : currentState.artworkUrl;

            if (!hasArtwork) {
                artworkUrl = null;
                artworkRequestId++;
                resetArtworkRetry();

                currentState.artworkVersion =
                    data.artwork_version;
            } else if (shouldAttemptArtwork) {
                const fetchedArtwork = await fetchArtwork(data);

                if (requestLifecycleId !== pageLifecycleId) {
                    if (fetchedArtwork) {
                        URL.revokeObjectURL(fetchedArtwork);
                    }

                    return;
                }

                if (fetchedArtwork) {
                    artworkUrl = fetchedArtwork;
                    currentState.artworkVersion =
                        data.artwork_version;
                }
            }

            if (requestLifecycleId !== pageLifecycleId) {
                if (
                    artworkUrl &&
                    artworkUrl !== currentState.artworkUrl
                ) {
                    URL.revokeObjectURL(artworkUrl);
                }

                return;
            }

            skin.updateTrack(
                data,
                artworkUrl,
                hasTrack
            );

            if (artworkUrl !== currentState.artworkUrl) {
                replaceArtworkUrl(artworkUrl);
            }

            currentState.title = data.title;
            currentState.artist = data.artist;
            hasInitializedTrackState = true;
        }

        updatePlaybackState(data, trackChanged);
    } catch (error) {
        if (requestLifecycleId === pageLifecycleId) {
            handleApiFailure(error);
        }
    } finally {
        isUpdating = false;
    }
}

// ─────────────────────────────────────────────
// Browser Source Lifecycle
// ─────────────────────────────────────────────

function startPolling() {
    if (pollingIntervalId !== null) {
        return;
    }

    pageLifecycleId++;

    updateNowPlaying();

    pollingIntervalId = setInterval(
        updateNowPlaying,
        POLL_INTERVAL_MS
    );
}

function stopPolling() {
    if (pollingIntervalId !== null) {
        clearInterval(pollingIntervalId);
        pollingIntervalId = null;
    }

    pageLifecycleId++;
    stopProgressAnimation();

    artworkRequestId++;

    if (currentState.artworkUrl) {
        URL.revokeObjectURL(currentState.artworkUrl);
        currentState.artworkUrl = null;
    }
}

window.addEventListener("pagehide", stopPolling);
window.addEventListener("pageshow", startPolling);

// ─────────────────────────────────────────────
// Initialization
// ─────────────────────────────────────────────

startPolling();