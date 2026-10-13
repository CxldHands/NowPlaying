 // ─────────────────────────────────────────────
 // Elements
 // ─────────────────────────────────────────────

const trackContent = document.getElementById("track-content");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const albumArt = document.getElementById("album-art");
const backgroundArt = document.getElementById("background-art");

const progress = document.getElementById("progress");
const position = document.getElementById("position");
const duration = document.getElementById("duration");

// ─────────────────────────────────────────────
// Track
// ─────────────────────────────────────────────

function updateTrack(data, artworkUrl, hasTrack) {
    if (!hasTrack) {
        title.textContent = "Waiting for music";
        artist.textContent = "Start playing something";
        albumArt.removeAttribute("src");
        backgroundArt.removeAttribute("src");

        return;
    }

    const trackChanged =
        data.title !== currentState.title ||
        data.artist !== currentState.artist;

    if (!trackChanged) {
        // Artwork may change independently of track metadata.
        if (artworkUrl) {
            albumArt.src = artworkUrl;
        } else {
            albumArt.removeAttribute("src");
        }

        return;
    }

    // Animate the current track out.
    gsap.to(trackContent, {
        opacity: 0,
        x: -40,
        duration: 0.25,
        ease: "power2.in",

        onComplete: () => {
            title.textContent = data.title ?? "No song playing";
            artist.textContent = data.artist ?? "—";

            if (artworkUrl) {
                albumArt.src = artworkUrl;
                backgroundArt.src = artworkUrl;
            } else {
                albumArt.removeAttribute("src");
                backgroundArt.removeAttribute("src");
            }

            gsap.set(trackContent, {
                x: 40,
            });

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

// ─────────────────────────────────────────────
// Skin Interface
// ─────────────────────────────────────────────

window.skin = {
    updateTrack,
    updateProgress,
};