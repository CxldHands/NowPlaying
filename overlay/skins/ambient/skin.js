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
// Track
// ─────────────────────────────────────────────

function updateTrack(data, artworkUrl, hasTrack) {
    if (!hasTrack) {
        title.textContent = "Waiting for music";
        artist.textContent = "Start playing something";
        albumArt.removeAttribute("src");
        return;
    }

    // Animate the current track out.
    gsap.to(trackContent, {
        opacity: 0,
        x: -30,
        duration: 0.2,
        ease: "power2.in",

        onComplete: () => {
            renderTrack(data, artworkUrl);

            gsap.set(trackContent, {
                x: 30,
            });

            gsap.to(trackContent, {
                opacity: 1,
                x: 0,
                duration: 0.3,
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

    if (artworkUrl) {
        albumArt.src = artworkUrl;
        backgroundArt.src = artworkUrl;
    }
}

// ─────────────────────────────────────────────
// Playback
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
// Skin Interface
// ─────────────────────────────────────────────

window.skin = {
    updateTrack,
    updateProgress,
};