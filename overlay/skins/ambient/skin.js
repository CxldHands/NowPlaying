// ─────────────────────────────────────────────
// Elements
// ─────────────────────────────────────────────

const trackContent = document.querySelector(".track-content");

const title = document.getElementById("title");
const artist = document.getElementById("artist");

const albumArt = document.getElementById("album-art");
const backgroundArt = document.getElementById("background-art");

const progress = document.getElementById("progress");
const position = document.getElementById("position");
const duration = document.getElementById("duration");

// ─────────────────────────────────────────────
// Transition State
// ─────────────────────────────────────────────

let trackTransitionId = 0;

// ─────────────────────────────────────────────
// Track
// ─────────────────────────────────────────────

function updateTrack(data, artworkUrl, hasTrack) {
    const transitionId = ++trackTransitionId;

    // Cancel previous transitions before handling the latest update.
    gsap.killTweensOf(trackContent);

    const prefersReducedMotion = window.matchMedia(
        "(prefers-reduced-motion: reduce)"
    ).matches;

    if (!hasTrack) {
        title.textContent = "Waiting for music";
        artist.textContent = "Start playing something";

        setArtwork(null);

        gsap.set(trackContent, {
            opacity: 1,
            x: 0,
        });

        resetProgress();

        return;
    }

    const nextTitle = data.title ?? "No song playing";
    const nextArtist = data.artist ?? "—";

    const trackChanged =
        nextTitle !== title.textContent ||
        nextArtist !== artist.textContent;

    // Keep both artwork layers synchronized when only artwork changes.
    if (!trackChanged) {
        setArtwork(artworkUrl);

        gsap.set(trackContent, {
            opacity: 1,
            x: 0,
        });

        return;
    }

    // Update immediately when reduced motion is preferred.
    if (prefersReducedMotion) {
        renderTrack(data, artworkUrl);

        gsap.set(trackContent, {
            opacity: 1,
            x: 0,
        });

        return;
    }

    // Animate the current track out.
    gsap.to(trackContent, {
        opacity: 0,
        x: -40,
        duration: 0.25,
        ease: "power2.in",

        onComplete: () => {
            // Ignore callbacks belonging to older transitions.
            if (transitionId !== trackTransitionId) {
                return;
            }

            renderTrack(data, artworkUrl);

            gsap.set(trackContent, {
                x: 40,
            });

            // Animate the new track into view.
            gsap.to(trackContent, {
                opacity: 1,
                x: 0,
                duration: 0.35,
                ease: "power2.out",
            });
        },
    });
}

// ─────────────────────────────────────────────
// Track Rendering
// ─────────────────────────────────────────────

function renderTrack(data, artworkUrl) {
    title.textContent = data.title ?? "No song playing";
    artist.textContent = data.artist ?? "—";

    setArtwork(artworkUrl);
}

function setArtwork(artworkUrl) {
    if (artworkUrl) {
        albumArt.src = artworkUrl;
        backgroundArt.src = artworkUrl;
    } else {
        albumArt.removeAttribute("src");
        backgroundArt.removeAttribute("src");
    }
}

// ─────────────────────────────────────────────
// Playback
// ─────────────────────────────────────────────

function updateProgress(data) {
    const positionMs = Math.max(0, data.position_ms);
    const durationMs = Math.max(0, data.duration_ms);

    position.textContent = formatTime(positionMs);
    duration.textContent = formatTime(durationMs);

    const percentage = durationMs > 0
        ? Math.min(100, (positionMs / durationMs) * 100)
        : 0;

    // Shared app.js handles playback interpolation.
    gsap.set(progress, {
        width: `${percentage}%`,
    });
}

function resetProgress() {
    position.textContent = formatTime(0);
    duration.textContent = formatTime(0);

    gsap.set(progress, {
        width: "0%",
    });
}

// ─────────────────────────────────────────────
// Skin Interface
// ─────────────────────────────────────────────

window.skin = {
    updateTrack,
    updateProgress,
};