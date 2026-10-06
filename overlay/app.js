const API_URL = "/now-playing";

const trackContent = document.getElementById("track-content");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const albumArt = document.getElementById("album-art");

const progress = document.getElementById("progress");
const position = document.getElementById("position");
const duration = document.getElementById("duration");

// ─────────────────────────────────────────────
// UI State
// ─────────────────────────────────────────────

// This represents what the UI currently knows.
// We compare incoming data against this state
// so we don't unnecessarily update the UI.
const currentState = {
    title: null,
    artist: null,
    artwork: null,
    isPlaying: false,
};

// ─────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────

function formatTime(milliseconds) {
    const seconds = Math.floor(milliseconds / 1000);

    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;

    return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

// ─────────────────────────────────────────────
// Track Updates
// ─────────────────────────────────────────────

function updateTrack(data) {
    const trackChanged =
        data.title !== currentState.title ||
        data.artist !== currentState.artist;

    if (!trackChanged) {
        return;
    }

    // Animate the current track out to the left.
    gsap.to(trackContent, {
        opacity: 0,
        x: -40,
        duration: 0.25,
        ease: "power2.in",

        onComplete: () => {
            title.textContent = data.title ?? "No song playing";
            artist.textContent = data.artist ?? "—";

            if (
                data.artwork &&
                data.artwork !== currentState.artwork
            ) {
                albumArt.src = data.artwork;
            }

            // Position the new track just to the right.
            gsap.set(trackContent, {
                x: 40,
            });

            // Bring the new track into place.
            gsap.to(trackContent, {
                opacity: 1,
                x: 0,
                duration: 0.35,
                ease: "power2.out",
            });

            currentState.title = data.title;
            currentState.artist = data.artist;
            currentState.artwork = data.artwork;
        },
    });
}

// ─────────────────────────────────────────────
// Playback State
// ─────────────────────────────────────────────

function updatePlaybackState(data) {
    if (data.is_playing === currentState.isPlaying) {
        return;
    }

    currentState.isPlaying = data.is_playing;

    console.log(
        currentState.isPlaying
            ? "Playback started"
            : "Playback paused"
    );
}

// ─────────────────────────────────────────────
// Progress
// ─────────────────────────────────────────────

function updateProgress(data) {
    position.textContent = formatTime(data.position_ms);
    duration.textContent = formatTime(data.duration_ms);

    if (data.duration_ms <= 0) {
        gsap.set(progress, {
            width: "0%",
        });

        return;
    }

    const percentage =
        (data.position_ms / data.duration_ms) * 100;

    gsap.to(progress, {
        width: `${percentage}%`,
        duration: 0.1,
        ease: "none",
    });
}

// ─────────────────────────────────────────────
// Main Update
// ─────────────────────────────────────────────

async function updateNowPlaying() {
    try {
        const response = await fetch(API_URL);
        const data = await response.json();

        updateTrack(data);
        updatePlaybackState(data);
        updateProgress(data);
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