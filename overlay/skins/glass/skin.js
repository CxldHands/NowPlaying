// ─────────────────────────────────────────────
// Elements
// ─────────────────────────────────────────────

const glassCard = document.getElementById("glass-card");
const trackContent = document.getElementById("track-content");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const albumArt = document.getElementById("album-art");

const progress = document.getElementById("progress");
const auraCore = document.querySelector(".aura-core");

// Length of the SVG progress path, used to calculate its dash offset.
const auraPathLength = 1000;

// ─────────────────────────────────────────────
// Transition State
// ─────────────────────────────────────────────

// Invalidates callbacks belonging to older track transitions.
let trackTransitionId = 0;

// ─────────────────────────────────────────────
// Track
// ─────────────────────────────────────────────

function updateTrack(data, artworkUrl, hasTrack) {
    const nextTitle = hasTrack
        ? (data.title ?? "No song playing")
        : "Waiting for music";

    const nextArtist = hasTrack
        ? (data.artist ?? "—")
        : "Start playing something";

    const prefersReducedMotion = window.matchMedia(
        "(prefers-reduced-motion: reduce)"
    ).matches;

    // Invalidate callbacks from previous updates.
    const transitionId = ++trackTransitionId;

    // Cancel existing transitions before processing the latest state.
    gsap.killTweensOf(trackContent);

    // Handle the unavailable state before the unchanged-metadata check.
    if (!hasTrack) {
        title.textContent = nextTitle;
        artist.textContent = nextArtist;

        setArtwork(null);
        resetProgress();

        gsap.set(trackContent, {
            opacity: 1,
            x: 0,
        });

        return;
    }

    // If metadata is unchanged, update artwork without a transition.
    if (
        title.textContent === nextTitle &&
        artist.textContent === nextArtist
    ) {
        setArtwork(artworkUrl);

        // Keep content visible if a previous transition was interrupted.
        gsap.set(trackContent, {
            opacity: 1,
            x: 0,
        });

        return;
    }

    // Update immediately when reduced motion is preferred.
    if (prefersReducedMotion) {
        title.textContent = nextTitle;
        artist.textContent = nextArtist;
        setArtwork(artworkUrl);

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
            // Ignore callbacks that belong to an older update.
            if (transitionId !== trackTransitionId) {
                return;
            }

            title.textContent = nextTitle;
            artist.textContent = nextArtist;
            setArtwork(artworkUrl);

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
// Artwork
// ─────────────────────────────────────────────

function setArtwork(artworkUrl) {
    if (artworkUrl) {
        albumArt.src = artworkUrl;
    } else {
        albumArt.removeAttribute("src");
    }
}

// ─────────────────────────────────────────────
// Playback Progress
// ─────────────────────────────────────────────

function updateProgress(data) {
    const durationMs = Math.max(
        0,
        Number(data.duration_ms) || 0
    );

    const positionMs = Math.max(
        0,
        Number(data.position_ms) || 0
    );

    const ratio = durationMs > 0
        ? Math.min(1, positionMs / durationMs)
        : 0;

    // SVG stroke-dashoffset controls the visible circular progress.
    const dashOffset = auraPathLength * (1 - ratio);

    progress.setAttribute("stroke-dashoffset", dashOffset);
    auraCore.setAttribute("stroke-dashoffset", dashOffset);
}

function resetProgress() {
    progress.setAttribute(
        "stroke-dashoffset",
        auraPathLength
    );

    auraCore.setAttribute(
        "stroke-dashoffset",
        auraPathLength
    );
}

// ─────────────────────────────────────────────
// Playback State
// ─────────────────────────────────────────────

function updatePlaybackState(data) {
    // CSS uses this class to style the card differently while paused.
    glassCard.classList.toggle("is-paused", !data.is_playing);
}

// ─────────────────────────────────────────────
// Skin Interface
// ─────────────────────────────────────────────

// The shared overlay controller calls these methods.
window.skin = {
    updateTrack,
    updateProgress,
    updatePlaybackState,
};